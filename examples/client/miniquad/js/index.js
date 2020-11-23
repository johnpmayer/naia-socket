
// Websocket stuff here implements hot reloading (RIP webpack-dev-server..)
const wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/livereload/';
conn = new WebSocket(wsUri);
conn.onclose = function() {
    setInterval(function() {
        let conn2 = new WebSocket(wsUri);
        conn2.onopen = function() {
            location.reload();
        };
    }, 500);
};

const naia_socket = {
    channel: null,
    encoder: new TextEncoder(),
    decoder: new TextDecoder("utf-8"),
    dropped_outgoing_messages: [],

    plugin: function (importObject) {
        importObject.env.connect = function (address) { naia_socket.connect(address); };
        importObject.env.send = function (message) { naia_socket.send(message); };
        importObject.env.resend_dropped_messages = function() { naia_socket.resend_dropped_messages(); };
    },

    connect: function (server_socket_address) {
        let _this = this;
        let server_socket_address_string = obj_manager.get_js_object(server_socket_address);
        let ADDRESS = "http://" + server_socket_address_string + "/new_rtc_session";

        let peer = new RTCPeerConnection({
            iceServers: [{
                urls: ["stun:stun.l.google.com:19302"]
            }]
        });

        this.channel = peer.createDataChannel("webudp", {
            ordered: false,
            maxRetransmits: 0
        });

        this.channel.binaryType = "arraybuffer";

        this.channel.onopen = function() {
            _this.channel.onmessage = function(evt) {
                let array = new Uint8Array(evt.data);
                let in_msg = _this.decoder.decode(array);
                interop.wasm_exports.receive(obj_manager.js_object(in_msg));
            };
        };

        this.channel.onerror = function(evt) {
            _this.error("data channel error", evt.message);
        };

        peer.onicecandidate = function(evt) {
            if (evt.candidate) {
                //console.log("received ice candidate", evt.candidate);
            } else {
                //console.log("all local candidates received");
            }
        };

        peer.createOffer().then(function(offer) {
            return peer.setLocalDescription(offer);
        }).then(function() {
            let request = new XMLHttpRequest();
            request.open("POST", ADDRESS);
            request.onload = function() {
                if (request.status === 200) {
                    let response = JSON.parse(request.responseText);
                    peer.setRemoteDescription(new RTCSessionDescription(response.answer)).then(function() {
                        let candidate = new RTCIceCandidate(response.candidate);
                        peer.addIceCandidate(candidate).then(function() {
                            //console.log("add ice candidate success");
                        }).catch(function(err) {
                            _this.error("error during 'addIceCandidate'", err);
                        });
                    }).catch(function(err) {
                        _this.error("error during 'setRemoteDescription'", err);
                    });
                } else {
                    _this.error("error sending POST /new_rtc_session request", { response_status: request.status });
                }
            };
            request.onerror = function(err) {
                _this.error("error sending POST /new_rtc_session request", err);
            };
            request.send(peer.localDescription.sdp);
        }).catch(function(err) {
            _this.error("error during 'createOffer'", err);
        });
    },

    error: function (desc, err) {
        err['naia_desc'] = desc;
        interop.wasm_exports.error(this.js_object(JSON.stringify(err)));
    },

    send_str: function (str) {
        if (this.channel) {
            try {
                this.channel.send(this.encoder.encode(str));
            }
            catch(err) {
                this.dropped_outgoing_messages.push(str);
            }
        }
        else {
            this.dropped_outgoing_messages.push(str);
        }
    },

    send: function (message) {
        let message_string = obj_manager.get_js_object(message);
        this.send_str(message_string);
    },

    resend_dropped_messages: function () {
        if (this.channel && this.dropped_outgoing_messages.length > 0) {
            let temp_array = this.dropped_outgoing_messages;
            this.dropped_outgoing_messages = [];
            for (let i = 0; i < temp_array.length; i+=1) {
                this.send_str(temp_array[i]);
            }
        }
    },
};

const obj_manager = {
    js_objects: {},
    unique_js_id: 0,

    plugin: function (importObject) {
        importObject.env.js_create_string = function (buf, max_len) { return obj_manager.js_create_string(buf, max_len); };
        importObject.env.js_unwrap_to_str = function (js_object, buf, max_len) { obj_manager.js_unwrap_to_str(js_object, buf, max_len); };
        importObject.env.js_string_length = function (js_object) { return obj_manager.js_string_length(js_object); };
        importObject.env.js_free_object = function (js_object) { obj_manager.js_free_object(js_object); };
    },

    js_create_string: function (buf, max_len) {
        let string = interop.UTF8ToString(buf, max_len);
        return this.js_object(string);
    },

    js_unwrap_to_str: function (js_object, buf, max_len) {
        let str = this.js_objects[js_object];
        let utf8array = this.toUTF8Array(str);
        let length = utf8array.length;
        let dest = new Uint8Array(interop.wasm_memory.buffer, buf, max_len);
        for (let i = 0; i < length; i++) {
            dest[i] = utf8array[i];
        }
    },

    js_string_length: function (js_object) {
        let str = this.js_objects[js_object];
        return this.toUTF8Array(str).length;
    },

    js_free_object: function (js_object) {
        delete this.js_objects[js_object];
    },

    toUTF8Array: function (str) {
        let utf8 = [];
        for (let i = 0; i < str.length; i++) {
            let charcode = str.charCodeAt(i);
            if (charcode < 0x80) utf8.push(charcode);
            else if (charcode < 0x800) {
                utf8.push(0xc0 | (charcode >> 6),
                    0x80 | (charcode & 0x3f));
            }
            else if (charcode < 0xd800 || charcode >= 0xe000) {
                utf8.push(0xe0 | (charcode >> 12),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            }
            // surrogate pair
            else {
                i++;
                // UTF-16 encodes 0x10000-0x10FFFF by
                // subtracting 0x10000 and splitting the
                // 20 bits of 0x0-0xFFFFF into two halves
                charcode = 0x10000 + (((charcode & 0x3ff) << 10)
                    | (str.charCodeAt(i) & 0x3ff))
                utf8.push(0xf0 | (charcode >> 18),
                    0x80 | ((charcode >> 12) & 0x3f),
                    0x80 | ((charcode >> 6) & 0x3f),
                    0x80 | (charcode & 0x3f));
            }
        }
        return utf8;
    },

    js_object: function (obj) {
        let id = this.unique_js_id;
        this.js_objects[id] = obj;
        this.unique_js_id += 1;
        return id;
    },

    get_js_object: function (id) {
        return this.js_objects[id];
    }
};

// Interop stuff
const interop = {
    wasm_memory: null,
    wasm_exports: null,
    importObject: {
        env: {}
    },
    plugins: [],

    plugin: function (importObject) {
        importObject.env.start_loop = function () { setInterval(function() { interop.wasm_exports.update(); }, 1); };
        importObject.env.js_log = function (message) { interop.js_log(message); };
    },

    js_log: function (message) {
        let message_string = obj_manager.get_js_object(message);
        console.log(message_string);
    },

    UTF8ToString: function (ptr, maxBytesToRead) {
        let u8Array = new Uint8Array(this.wasm_memory.buffer, ptr);
        let idx = 0;
        let endIdx = idx + maxBytesToRead;
        let str = '';
        while (!(idx >= endIdx)) {
            let u0 = u8Array[idx++];
            if (!u0) return str;

            if (!(u0 & 0x80)) { str += String.fromCharCode(u0); continue; }
            let u1 = u8Array[idx++] & 63;
            if ((u0 & 0xE0) === 0xC0) { str += String.fromCharCode(((u0 & 31) << 6) | u1); continue; }
            let u2 = u8Array[idx++] & 63;
            if ((u0 & 0xF0) === 0xE0) {
                u0 = ((u0 & 15) << 12) | (u1 << 6) | u2;
            } else {
                if ((u0 & 0xF8) !== 0xF0) console.warn('Invalid UTF-8 leading byte 0x' + u0.toString(16) + ' encountered when deserializing a UTF-8 string on the asm.js/wasm heap to a JS string!');
                u0 = ((u0 & 7) << 18) | (u1 << 12) | (u2 << 6) | (u8Array[idx++] & 63);
            }
            if (u0 < 0x10000) {
                str += String.fromCharCode(u0);
            } else {
                var ch = u0 - 0x10000;
                str += String.fromCharCode(0xD800 | (ch >> 10), 0xDC00 | (ch & 0x3FF));
            }
        }
        return str;
    },

    register_plugins: function () {
        if (this.plugins === undefined)
            return;

        for (let i = 0; i < this.plugins.length; i++) {
            if (this.plugins[i].register_plugin !== undefined && this.plugins[i].register_plugin != null) {
                this.plugins[i].register_plugin(this.importObject);
            }
        }
    },

    init_plugins: function () {
        if (this.plugins === undefined)
            return;

        for (let i = 0; i < this.plugins.length; i++) {
            if (this.plugins[i].on_init !== undefined && this.plugins[i].on_init != null) {
                this.plugins[i].on_init();
            }
        }
    },

    add_plugin: function (plugin) {
        this.plugins.push(plugin);
    },

    load: function (wasm_path) {
        const _this = this;
        let req = fetch(wasm_path);

        this.register_plugins();

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            WebAssembly.instantiateStreaming(req, this.importObject)
                .then(obj => {
                    _this.wasm_memory = obj.instance.exports.memory;
                    _this.wasm_exports = obj.instance.exports;
                    _this.init_plugins();
                    obj.instance.exports.main();
                });
        } else {
            req
                .then(function (x) { return x.arrayBuffer(); })
                .then(function (bytes) { return WebAssembly.instantiate(bytes, _this.importObject); })
                .then(function (obj) {
                    _this.wasm_memory = obj.instance.exports.memory;
                    _this.wasm_exports = obj.instance.exports;
                    _this.init_plugins();
                    obj.instance.exports.main();
                });
        }
    }
};

interop.add_plugin({ register_plugin: interop.plugin });
interop.add_plugin({ register_plugin: obj_manager.plugin });
interop.add_plugin({ register_plugin: naia_socket.plugin });