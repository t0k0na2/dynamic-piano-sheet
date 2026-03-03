const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

let WASM_VECTOR_LEN = 0;

const __wbindgen_enum_BiquadFilterType = ["lowpass", "highpass", "bandpass", "lowshelf", "highshelf", "peaking", "notch", "allpass"];

const __wbindgen_enum_OscillatorType = ["sine", "square", "sawtooth", "triangle", "custom"];

const MidiPlayerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_midiplayer_free(ptr >>> 0, 1));

export class MidiPlayer {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MidiPlayer.prototype);
        obj.__wbg_ptr = ptr;
        MidiPlayerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MidiPlayerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_midiplayer_free(ptr, 0);
    }
    /**
     * @param {number} volume
     */
    set_volume(volume) {
        wasm.midiplayer_set_volume(this.__wbg_ptr, volume);
    }
    /**
     * @returns {number}
     */
    current_bar() {
        const ret = wasm.midiplayer_current_bar(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    song_length() {
        const ret = wasm.midiplayer_song_length(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} start_bar
     * @param {number} end_bar
     */
    set_loop_bars(start_bar, end_bar) {
        wasm.midiplayer_set_loop_bars(this.__wbg_ptr, start_bar, end_bar);
    }
    /**
     * @param {Uint8Array} bin
     */
    load_soundfont(bin) {
        const ptr0 = passArray8ToWasm0(bin, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.midiplayer_load_soundfont(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} range_sec
     */
    set_display_range(range_sec) {
        wasm.midiplayer_set_display_range(this.__wbg_ptr, range_sec);
    }
    /**
     * @returns {number}
     */
    current_playback_time() {
        const ret = wasm.midiplayer_current_playback_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {MidiPlayer}
     */
    static new() {
        const ret = wasm.midiplayer_new();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return MidiPlayer.__wrap(ret[0]);
    }
    play() {
        wasm.midiplayer_play(this.__wbg_ptr);
    }
    /**
     * @param {number} delta
     * @param {boolean} clear_sounds
     */
    skip(delta, clear_sounds) {
        wasm.midiplayer_skip(this.__wbg_ptr, delta, clear_sounds);
    }
    stop() {
        wasm.midiplayer_stop(this.__wbg_ptr);
    }
    /**
     * @param {number} delta_time
     */
    tick(delta_time) {
        const ret = wasm.midiplayer_tick(this.__wbg_ptr, delta_time);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {boolean}
     */
    ready() {
        const ret = wasm.midiplayer_ready(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {CanvasRenderingContext2D} context
     * @param {number} left
     * @param {number} top
     * @param {number} width
     * @param {number} height
     */
    render(context, left, top, width, height) {
        const ret = wasm.midiplayer_render(this.__wbg_ptr, context, left, top, width, height);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @returns {number}
     */
    volume() {
        const ret = wasm.midiplayer_volume(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    num_bars() {
        const ret = wasm.midiplayer_num_bars(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} bar
     * @param {boolean} clear_sounds
     */
    seek_bar(bar, clear_sounds) {
        wasm.midiplayer_seek_bar(this.__wbg_ptr, bar, clear_sounds);
    }
    /**
     * @param {Uint8Array} bin
     */
    load_midi(bin) {
        const ptr0 = passArray8ToWasm0(bin, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.midiplayer_load_midi(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} time
     * @param {boolean} clear_sounds
     */
    seek_time(time, clear_sounds) {
        wasm.midiplayer_seek_time(this.__wbg_ptr, time, clear_sounds);
    }
}
if (Symbol.dispose) MidiPlayer.prototype[Symbol.dispose] = MidiPlayer.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Q_83917c4ef2b50b55 = function(arg0) {
        const ret = arg0.Q;
        return ret;
    };
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_beginPath_08eae248f93ea32d = function(arg0) {
        arg0.beginPath();
    };
    imports.wbg.__wbg_cancelScheduledValues_569985df1872064b = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.cancelScheduledValues(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_connect_c49933915e0ca61d = function() { return handleError(function (arg0, arg1) {
        arg0.connect(arg1);
    }, arguments) };
    imports.wbg.__wbg_connect_f28a2db518e02462 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.connect(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_copyToChannel_30d90303302ec449 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.copyToChannel(getArrayF32FromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_createBiquadFilter_7f1bc4b6ddbb5b54 = function() { return handleError(function (arg0) {
        const ret = arg0.createBiquadFilter();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createBufferSource_e690bdc1d3edcfdc = function() { return handleError(function (arg0) {
        const ret = arg0.createBufferSource();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createBuffer_386b4ff6efbb358e = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.createBuffer(arg1 >>> 0, arg2 >>> 0, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createBuffer_ca77f7a43c0db647 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.createBuffer(arg1 >>> 0, arg2 >>> 0, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createConvolver_e842444277560123 = function() { return handleError(function (arg0) {
        const ret = arg0.createConvolver();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createDelay_6439a64d02500fa1 = function() { return handleError(function (arg0) {
        const ret = arg0.createDelay();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createGain_704a1ee093f832bf = function() { return handleError(function (arg0) {
        const ret = arg0.createGain();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createGain_d5704df14f1e271f = function() { return handleError(function (arg0) {
        const ret = arg0.createGain();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createOscillator_21319c8981a0df27 = function() { return handleError(function (arg0) {
        const ret = arg0.createOscillator();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createOscillator_d01e2306cd874562 = function() { return handleError(function (arg0) {
        const ret = arg0.createOscillator();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_createStereoPanner_1be80152de6c9e4a = function() { return handleError(function (arg0) {
        const ret = arg0.createStereoPanner();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_currentTime_7d6df02e29507923 = function(arg0) {
        const ret = arg0.currentTime;
        return ret;
    };
    imports.wbg.__wbg_delayTime_56315e7f0ef53379 = function(arg0) {
        const ret = arg0.delayTime;
        return ret;
    };
    imports.wbg.__wbg_destination_1dd37feba0c0cab6 = function(arg0) {
        const ret = arg0.destination;
        return ret;
    };
    imports.wbg.__wbg_detune_93917c927486dee3 = function(arg0) {
        const ret = arg0.detune;
        return ret;
    };
    imports.wbg.__wbg_detune_dd5606c8f137d329 = function(arg0) {
        const ret = arg0.detune;
        return ret;
    };
    imports.wbg.__wbg_disconnect_73648182b9afde22 = function() { return handleError(function (arg0) {
        arg0.disconnect();
    }, arguments) };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_exponentialRampToValueAtTime_ef83b0a5a912746d = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.exponentialRampToValueAtTime(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_fillRect_84131220403e26a4 = function(arg0, arg1, arg2, arg3, arg4) {
        arg0.fillRect(arg1, arg2, arg3, arg4);
    };
    imports.wbg.__wbg_fillText_56566d8049e84e17 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
    }, arguments) };
    imports.wbg.__wbg_fill_dd0f756eea36e037 = function(arg0) {
        arg0.fill();
    };
    imports.wbg.__wbg_frequency_2ecf1d4e0db76b25 = function(arg0) {
        const ret = arg0.frequency;
        return ret;
    };
    imports.wbg.__wbg_frequency_e628b4f5b2573bed = function(arg0) {
        const ret = arg0.frequency;
        return ret;
    };
    imports.wbg.__wbg_gain_435505e65fb96146 = function(arg0) {
        const ret = arg0.gain;
        return ret;
    };
    imports.wbg.__wbg_lineTo_4b884d8cebfc8c54 = function(arg0, arg1, arg2) {
        arg0.lineTo(arg1, arg2);
    };
    imports.wbg.__wbg_linearRampToValueAtTime_4e10412ad6091d32 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.linearRampToValueAtTime(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_log_1d990106d99dacb7 = function(arg0) {
        console.log(arg0);
    };
    imports.wbg.__wbg_moveTo_36127921f1ca46a5 = function(arg0, arg1, arg2) {
        arg0.moveTo(arg1, arg2);
    };
    imports.wbg.__wbg_new_5e542c992f14cb6f = function() { return handleError(function () {
        const ret = new lAudioContext();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_pan_35871c6690f4b68e = function(arg0) {
        const ret = arg0.pan;
        return ret;
    };
    imports.wbg.__wbg_playbackRate_422af96f0434a11e = function(arg0) {
        const ret = arg0.playbackRate;
        return ret;
    };
    imports.wbg.__wbg_random_cc1f9237d866d212 = function() {
        const ret = Math.random();
        return ret;
    };
    imports.wbg.__wbg_roundRect_35915ab812b6013d = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        arg0.roundRect(arg1, arg2, arg3, arg4, arg5);
    }, arguments) };
    imports.wbg.__wbg_sampleRate_9a34c7246da700ce = function(arg0) {
        const ret = arg0.sampleRate;
        return ret;
    };
    imports.wbg.__wbg_sampleRate_e69ffe52a3b9a888 = function(arg0) {
        const ret = arg0.sampleRate;
        return ret;
    };
    imports.wbg.__wbg_setTargetAtTime_46a2f5babac83293 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.setTargetAtTime(arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setValueAtTime_c7e9a87ece93a066 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.setValueAtTime(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_set_buffer_25c70ce663d1667c = function(arg0, arg1) {
        arg0.buffer = arg1;
    };
    imports.wbg.__wbg_set_buffer_50187dffee006fe0 = function(arg0, arg1) {
        arg0.buffer = arg1;
    };
    imports.wbg.__wbg_set_fillStyle_c9a0550307cd4671 = function(arg0, arg1, arg2) {
        arg0.fillStyle = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_font_37c5ab71d0189314 = function(arg0, arg1, arg2) {
        arg0.font = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_loopEnd_59f552d8f7c43f99 = function(arg0, arg1) {
        arg0.loopEnd = arg1;
    };
    imports.wbg.__wbg_set_loopStart_b36eee7edf35abfb = function(arg0, arg1) {
        arg0.loopStart = arg1;
    };
    imports.wbg.__wbg_set_loop_8096808d7e93a3af = function(arg0, arg1) {
        arg0.loop = arg1 !== 0;
    };
    imports.wbg.__wbg_set_strokeStyle_697a576d2d3fbeaa = function(arg0, arg1, arg2) {
        arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_textAlign_5d82eb01e9d2291e = function(arg0, arg1, arg2) {
        arg0.textAlign = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_textBaseline_9e8ed61033c5023d = function(arg0, arg1, arg2) {
        arg0.textBaseline = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_set_type_55ae98ed238be327 = function(arg0, arg1) {
        arg0.type = __wbindgen_enum_BiquadFilterType[arg1];
    };
    imports.wbg.__wbg_set_type_94f7a29ccbd3cd38 = function(arg0, arg1) {
        arg0.type = __wbindgen_enum_OscillatorType[arg1];
    };
    imports.wbg.__wbg_set_value_9b29412dd286d5d4 = function(arg0, arg1) {
        arg0.value = arg1;
    };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_start_7c1e021f6b191cad = function() { return handleError(function (arg0, arg1) {
        arg0.start(arg1);
    }, arguments) };
    imports.wbg.__wbg_start_865ed2d65a6d353e = function() { return handleError(function (arg0, arg1) {
        arg0.start(arg1);
    }, arguments) };
    imports.wbg.__wbg_start_958fabbec6509da5 = function() { return handleError(function (arg0) {
        arg0.start();
    }, arguments) };
    imports.wbg.__wbg_stop_47ec9f3363a59206 = function() { return handleError(function (arg0, arg1) {
        arg0.stop(arg1);
    }, arguments) };
    imports.wbg.__wbg_stop_8e9b036871dbd86b = function() { return handleError(function (arg0, arg1) {
        arg0.stop(arg1);
    }, arguments) };
    imports.wbg.__wbg_stroke_a18b81eb49ff370e = function(arg0) {
        arg0.stroke();
    };
    imports.wbg.__wbg_value_1424af07c44a199e = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('dynamic_piano_sheet_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
