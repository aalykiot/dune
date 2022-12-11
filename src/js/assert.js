// Currently in sync with Node.js lib/assert.js
// https://github.com/nodejs/node/commit/2a51ae424a513ec9a6aa3466baa0cc1d55dd4f3b

// Originally from narwhal.js (http://narwhaljs.org)
// Copyright (c) 2009 Thomas Robinson <280north.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the 'Software'), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED 'AS IS', WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

var P = Object.create;
var d = Object.defineProperty;
var T = Object.getOwnPropertyDescriptor;
var F = Object.getOwnPropertyNames;
var I = Object.getPrototypeOf, K = Object.prototype.hasOwnProperty;
var o = (t, e)=>d(t, "name", {
        value: e,
        configurable: !0
    });
var W = (t, e)=>()=>(e || t((e = {
            exports: {}
        }).exports, e), e.exports);
var $ = (t, e, n, r)=>{
    if (e && typeof e == "object" || typeof e == "function") for (let i of F(e))!K.call(t, i) && i !== n && d(t, i, {
        get: ()=>e[i],
        enumerable: !(r = T(e, i)) || r.enumerable
    });
    return t;
};
var y = (t, e, n)=>(n = t != null ? P(I(t)) : {}, $(e || !t || !t.__esModule ? d(n, "default", {
        value: t,
        enumerable: !0
    }) : n, t));
var m = W((J, h)=>{
    "use strict";
    var c = typeof Reflect == "object" ? Reflect : null, g = c && typeof c.apply == "function" ? c.apply : o(function(e, n, r) {
        return Function.prototype.apply.call(e, n, r);
    }, "ReflectApply"), v;
    c && typeof c.ownKeys == "function" ? v = c.ownKeys : Object.getOwnPropertySymbols ? v = o(function(e) {
        return Object.getOwnPropertyNames(e).concat(Object.getOwnPropertySymbols(e));
    }, "ReflectOwnKeys") : v = o(function(e) {
        return Object.getOwnPropertyNames(e);
    }, "ReflectOwnKeys");
    function S(t) {
        console && console.warn && console.warn(t);
    }
    o(S, "ProcessEmitWarning");
    var w = Number.isNaN || o(function(e) {
        return e !== e;
    }, "NumberIsNaN");
    function f() {
        f.init.call(this);
    }
    o(f, "EventEmitter");
    h.exports = f;
    h.exports.once = q;
    f.EventEmitter = f;
    f.prototype._events = void 0;
    f.prototype._eventsCount = 0;
    f.prototype._maxListeners = void 0;
    var _ = 10;
    function p(t) {
        if (typeof t != "function") throw new TypeError('The "listener" argument must be of type Function. Received type ' + typeof t);
    }
    o(p, "checkListener");
    Object.defineProperty(f, "defaultMaxListeners", {
        enumerable: !0,
        get: function() {
            return _;
        },
        set: function(t) {
            if (typeof t != "number" || t < 0 || w(t)) throw new RangeError('The value of "defaultMaxListeners" is out of range. It must be a non-negative number. Received ' + t + ".");
            _ = t;
        }
    });
    f.init = function() {
        (this._events === void 0 || this._events === Object.getPrototypeOf(this)._events) && (this._events = Object.create(null), this._eventsCount = 0), this._maxListeners = this._maxListeners || void 0;
    };
    f.prototype.setMaxListeners = o(function(e) {
        if (typeof e != "number" || e < 0 || w(e)) throw new RangeError('The value of "n" is out of range. It must be a non-negative number. Received ' + e + ".");
        return this._maxListeners = e, this;
    }, "setMaxListeners");
    function b(t) {
        return t._maxListeners === void 0 ? f.defaultMaxListeners : t._maxListeners;
    }
    o(b, "_getMaxListeners");
    f.prototype.getMaxListeners = o(function() {
        return b(this);
    }, "getMaxListeners");
    f.prototype.emit = o(function(e) {
        for(var n = [], r = 1; r < arguments.length; r++)n.push(arguments[r]);
        var i = e === "error", u = this._events;
        if (u !== void 0) i = i && u.error === void 0;
        else if (!i) return !1;
        if (i) {
            var s;
            if (n.length > 0 && (s = n[0]), s instanceof Error) throw s;
            var a = new Error("Unhandled error." + (s ? " (" + s.message + ")" : ""));
            throw a.context = s, a;
        }
        var l = u[e];
        if (l === void 0) return !1;
        if (typeof l == "function") g(l, this, n);
        else for(var L = l.length, A = j(l, L), r = 0; r < L; ++r)g(A[r], this, n);
        return !0;
    }, "emit");
    function E(t, e, n, r) {
        var i, u, s;
        if (p(n), u = t._events, u === void 0 ? (u = t._events = Object.create(null), t._eventsCount = 0) : (u.newListener !== void 0 && (t.emit("newListener", e, n.listener ? n.listener : n), u = t._events), s = u[e]), s === void 0) s = u[e] = n, ++t._eventsCount;
        else if (typeof s == "function" ? s = u[e] = r ? [
            n,
            s
        ] : [
            s,
            n
        ] : r ? s.unshift(n) : s.push(n), i = b(t), i > 0 && s.length > i && !s.warned) {
            s.warned = !0;
            var a = new Error("Possible EventEmitter memory leak detected. " + s.length + " " + String(e) + " listeners added. Use emitter.setMaxListeners() to increase limit");
            a.name = "MaxListenersExceededWarning", a.emitter = t, a.type = e, a.count = s.length, S(a);
        }
        return t;
    }
    o(E, "_addListener");
    f.prototype.addListener = o(function(e, n) {
        return E(this, e, n, !1);
    }, "addListener");
    f.prototype.on = f.prototype.addListener;
    f.prototype.prependListener = o(function(e, n) {
        return E(this, e, n, !0);
    }, "prependListener");
    function U() {
        if (!this.fired) return this.target.removeListener(this.type, this.wrapFn), this.fired = !0, arguments.length === 0 ? this.listener.call(this.target) : this.listener.apply(this.target, arguments);
    }
    o(U, "onceWrapper");
    function O(t, e, n) {
        var r = {
            fired: !1,
            wrapFn: void 0,
            target: t,
            type: e,
            listener: n
        }, i = U.bind(r);
        return i.listener = n, r.wrapFn = i, i;
    }
    o(O, "_onceWrap");
    f.prototype.once = o(function(e, n) {
        return p(n), this.on(e, O(this, e, n)), this;
    }, "once");
    f.prototype.prependOnceListener = o(function(e, n) {
        return p(n), this.prependListener(e, O(this, e, n)), this;
    }, "prependOnceListener");
    f.prototype.removeListener = o(function(e, n) {
        var r, i, u, s, a;
        if (p(n), i = this._events, i === void 0) return this;
        if (r = i[e], r === void 0) return this;
        if (r === n || r.listener === n) --this._eventsCount === 0 ? this._events = Object.create(null) : (delete i[e], i.removeListener && this.emit("removeListener", e, r.listener || n));
        else if (typeof r != "function") {
            for(u = -1, s = r.length - 1; s >= 0; s--)if (r[s] === n || r[s].listener === n) {
                a = r[s].listener, u = s;
                break;
            }
            if (u < 0) return this;
            u === 0 ? r.shift() : k(r, u), r.length === 1 && (i[e] = r[0]), i.removeListener !== void 0 && this.emit("removeListener", e, a || n);
        }
        return this;
    }, "removeListener");
    f.prototype.off = f.prototype.removeListener;
    f.prototype.removeAllListeners = o(function(e) {
        var n, r, i;
        if (r = this._events, r === void 0) return this;
        if (r.removeListener === void 0) return arguments.length === 0 ? (this._events = Object.create(null), this._eventsCount = 0) : r[e] !== void 0 && (--this._eventsCount === 0 ? this._events = Object.create(null) : delete r[e]), this;
        if (arguments.length === 0) {
            var u = Object.keys(r), s;
            for(i = 0; i < u.length; ++i)s = u[i], s !== "removeListener" && this.removeAllListeners(s);
            return this.removeAllListeners("removeListener"), this._events = Object.create(null), this._eventsCount = 0, this;
        }
        if (n = r[e], typeof n == "function") this.removeListener(e, n);
        else if (n !== void 0) for(i = n.length - 1; i >= 0; i--)this.removeListener(e, n[i]);
        return this;
    }, "removeAllListeners");
    function x(t, e, n) {
        var r = t._events;
        if (r === void 0) return [];
        var i = r[e];
        return i === void 0 ? [] : typeof i == "function" ? n ? [
            i.listener || i
        ] : [
            i
        ] : n ? H(i) : j(i, i.length);
    }
    o(x, "_listeners");
    f.prototype.listeners = o(function(e) {
        return x(this, e, !0);
    }, "listeners");
    f.prototype.rawListeners = o(function(e) {
        return x(this, e, !1);
    }, "rawListeners");
    f.listenerCount = function(t, e) {
        return typeof t.listenerCount == "function" ? t.listenerCount(e) : C.call(t, e);
    };
    f.prototype.listenerCount = C;
    function C(t) {
        var e = this._events;
        if (e !== void 0) {
            var n = e[t];
            if (typeof n == "function") return 1;
            if (n !== void 0) return n.length;
        }
        return 0;
    }
    o(C, "listenerCount");
    f.prototype.eventNames = o(function() {
        return this._eventsCount > 0 ? v(this._events) : [];
    }, "eventNames");
    function j(t, e) {
        for(var n = new Array(e), r = 0; r < e; ++r)n[r] = t[r];
        return n;
    }
    o(j, "arrayClone");
    function k(t, e) {
        for(; e + 1 < t.length; e++)t[e] = t[e + 1];
        t.pop();
    }
    o(k, "spliceOne");
    function H(t) {
        for(var e = new Array(t.length), n = 0; n < e.length; ++n)e[n] = t[n].listener || t[n];
        return e;
    }
    o(H, "unwrapListeners");
    function q(t, e) {
        return new Promise(function(n, r) {
            function i(s) {
                t.removeListener(e, u), r(s);
            }
            o(i, "errorListener");
            function u() {
                typeof t.removeListener == "function" && t.removeListener("error", i), n([].slice.call(arguments));
            }
            o(u, "resolver"), R(t, e, u, {
                once: !0
            }), e !== "error" && z(t, i, {
                once: !0
            });
        });
    }
    o(q, "once");
    function z(t, e, n) {
        typeof t.on == "function" && R(t, "error", e, n);
    }
    o(z, "addErrorHandlerIfEventEmitter");
    function R(t, e, n, r) {
        if (typeof t.on == "function") r.once ? t.once(e, n) : t.on(e, n);
        else if (typeof t.addEventListener == "function") t.addEventListener(e, o(function i(u) {
            r.once && t.removeEventListener(e, i), n(u);
        }, "wrapListener"));
        else throw new TypeError('The "emitter" argument must be of type EventEmitter. Received type ' + typeof t);
    }
    o(R, "eventTargetAgnosticAddListener");
});
var N = y(m()), M = y(m()), { EventEmitter: Q , init: V , listenerCount: X , once: Y  } = M, { default: B , ...D } = M, Z = N.default ?? B ?? D;
const events = new Q();
events.setMaxListeners(1 << 10);
const deno = typeof Deno !== "undefined";
const __default = {
    title: deno ? "deno" : "browser",
    browser: true,
    env: deno ? new Proxy({}, {
        get (_target, prop) {
            return Deno.env.get(String(prop));
        },
        ownKeys: ()=>Reflect.ownKeys(Deno.env.toObject()),
        getOwnPropertyDescriptor: (_target, name)=>{
            const e = Deno.env.toObject();
            if (name in Deno.env.toObject()) {
                const o = {
                    enumerable: true,
                    configurable: true
                };
                if (typeof name === "string") {
                    o.value = e[name];
                }
                return o;
            }
        },
        set (_target, prop, value) {
            Deno.env.set(String(prop), String(value));
            return value;
        }
    }) : {},
    argv: deno ? Deno.args ?? [] : [],
    pid: deno ? Deno.pid ?? 0 : 0,
    version: "v16.18.0",
    versions: {
        node: '16.18.0',
        v8: '9.4.146.26-node.22',
        uv: '1.43.0',
        zlib: '1.2.11',
        brotli: '1.0.9',
        ares: '1.18.1',
        modules: '93',
        nghttp2: '1.47.0',
        napi: '8',
        llhttp: '6.0.10',
        openssl: '1.1.1q+quic',
        cldr: '41.0',
        icu: '71.1',
        tz: '2022b',
        unicode: '14.0',
        ngtcp2: '0.8.1',
        nghttp3: '0.7.0',
        ...deno ? Deno.version ?? {} : {}
    },
    on: (...args)=>events.on(...args),
    addListener: (...args)=>events.addListener(...args),
    once: (...args)=>events.once(...args),
    off: (...args)=>events.off(...args),
    removeListener: (...args)=>events.removeListener(...args),
    removeAllListeners: (...args)=>events.removeAllListeners(...args),
    emit: (...args)=>events.emit(...args),
    prependListener: (...args)=>events.prependListener(...args),
    prependOnceListener: (...args)=>events.prependOnceListener(...args),
    listeners: ()=>[],
    emitWarning: ()=>{
        throw new Error("process.emitWarning is not supported");
    },
    binding: ()=>{
        throw new Error("process.binding is not supported");
    },
    cwd: ()=>deno ? Deno.cwd?.() ?? "/" : "/",
    chdir: (path)=>{
        if (deno) {
            Deno.chdir(path);
        } else {
            throw new Error("process.chdir is not supported");
        }
    },
    umask: ()=>deno ? Deno.umask ?? 0 : 0,
    nextTick: (func, ...args)=>queueMicrotask(()=>func(...args))
};
var __global$ = globalThis || (typeof window !== "undefined" ? window : self);
var pt = Object.create;
var _r = Object.defineProperty;
var lt = Object.getOwnPropertyDescriptor;
var gt = Object.getOwnPropertyNames, Gr = Object.getOwnPropertySymbols, dt = Object.getPrototypeOf, Wr = Object.prototype.hasOwnProperty, bt = Object.prototype.propertyIsEnumerable;
var zr = (r, e)=>{
    var t = {};
    for(var n in r)Wr.call(r, n) && e.indexOf(n) < 0 && (t[n] = r[n]);
    if (r != null && Gr) for (var n of Gr(r))e.indexOf(n) < 0 && bt.call(r, n) && (t[n] = r[n]);
    return t;
};
var p = (r, e)=>()=>(e || r((e = {
            exports: {}
        }).exports, e), e.exports);
var mt = (r, e, t, n)=>{
    if (e && typeof e == "object" || typeof e == "function") for (let o of gt(e))!Wr.call(r, o) && o !== t && _r(r, o, {
        get: ()=>e[o],
        enumerable: !(n = lt(e, o)) || n.enumerable
    });
    return r;
};
var At = (r, e, t)=>(t = r != null ? pt(dt(r)) : {}, mt(e || !r || !r.__esModule ? _r(t, "default", {
        value: r,
        enumerable: !0
    }) : t, r));
var pr = p((po, Vr)=>{
    "use strict";
    Vr.exports = function() {
        if (typeof Symbol != "function" || typeof Object.getOwnPropertySymbols != "function") return !1;
        if (typeof Symbol.iterator == "symbol") return !0;
        var e = {}, t = Symbol("test"), n = Object(t);
        if (typeof t == "string" || Object.prototype.toString.call(t) !== "[object Symbol]" || Object.prototype.toString.call(n) !== "[object Symbol]") return !1;
        var o = 42;
        e[t] = o;
        for(t in e)return !1;
        if (typeof Object.keys == "function" && Object.keys(e).length !== 0 || typeof Object.getOwnPropertyNames == "function" && Object.getOwnPropertyNames(e).length !== 0) return !1;
        var i = Object.getOwnPropertySymbols(e);
        if (i.length !== 1 || i[0] !== t || !Object.prototype.propertyIsEnumerable.call(e, t)) return !1;
        if (typeof Object.getOwnPropertyDescriptor == "function") {
            var a = Object.getOwnPropertyDescriptor(e, t);
            if (a.value !== o || a.enumerable !== !0) return !1;
        }
        return !0;
    };
});
var x = p((lo, Jr)=>{
    "use strict";
    var St = pr();
    Jr.exports = function() {
        return St() && !!Symbol.toStringTag;
    };
});
var Zr = p((go, Hr)=>{
    "use strict";
    var Lr = typeof Symbol != "undefined" && Symbol, ht = pr();
    Hr.exports = function() {
        return typeof Lr != "function" || typeof Symbol != "function" || typeof Lr("foo") != "symbol" || typeof Symbol("bar") != "symbol" ? !1 : ht();
    };
});
var Kr = p((bo, Yr)=>{
    "use strict";
    var vt = "Function.prototype.bind called on incompatible ", lr = Array.prototype.slice, Ot = Object.prototype.toString, jt = "[object Function]";
    Yr.exports = function(e) {
        var t = this;
        if (typeof t != "function" || Ot.call(t) !== jt) throw new TypeError(vt + t);
        for(var n = lr.call(arguments, 1), o, i = function() {
            if (this instanceof o) {
                var g = t.apply(this, n.concat(lr.call(arguments)));
                return Object(g) === g ? g : this;
            } else return t.apply(e, n.concat(lr.call(arguments)));
        }, a = Math.max(0, t.length - n.length), f = [], y = 0; y < a; y++)f.push("$" + y);
        if (o = Function("binder", "return function (" + f.join(",") + "){ return binder.apply(this,arguments); }")(i), t.prototype) {
            var l = function() {};
            l.prototype = t.prototype, o.prototype = new l, l.prototype = null;
        }
        return o;
    };
});
var W1 = p((mo, Qr)=>{
    "use strict";
    var Pt = Kr();
    Qr.exports = Function.prototype.bind || Pt;
});
var re = p((Ao, Xr)=>{
    "use strict";
    var wt = W1();
    Xr.exports = wt.call(Function.call, Object.prototype.hasOwnProperty);
});
var J = p((So, oe)=>{
    "use strict";
    var c, B = SyntaxError, ne = Function, F = TypeError, gr = function(r) {
        try {
            return ne('"use strict"; return (' + r + ").constructor;")();
        } catch (e) {}
    }, v = Object.getOwnPropertyDescriptor;
    if (v) try {
        v({}, "");
    } catch (r) {
        v = null;
    }
    var dr = function() {
        throw new F;
    }, Et = v ? function() {
        try {
            return arguments.callee, dr;
        } catch (r) {
            try {
                return v(arguments, "callee").get;
            } catch (e) {
                return dr;
            }
        }
    }() : dr, E = Zr()(), A = Object.getPrototypeOf || function(r) {
        return r.__proto__;
    }, T = {}, Tt = typeof Uint8Array == "undefined" ? c : A(Uint8Array), I = {
        "%AggregateError%": typeof AggregateError == "undefined" ? c : AggregateError,
        "%Array%": Array,
        "%ArrayBuffer%": typeof ArrayBuffer == "undefined" ? c : ArrayBuffer,
        "%ArrayIteratorPrototype%": E ? A([][Symbol.iterator]()) : c,
        "%AsyncFromSyncIteratorPrototype%": c,
        "%AsyncFunction%": T,
        "%AsyncGenerator%": T,
        "%AsyncGeneratorFunction%": T,
        "%AsyncIteratorPrototype%": T,
        "%Atomics%": typeof Atomics == "undefined" ? c : Atomics,
        "%BigInt%": typeof BigInt == "undefined" ? c : BigInt,
        "%Boolean%": Boolean,
        "%DataView%": typeof DataView == "undefined" ? c : DataView,
        "%Date%": Date,
        "%decodeURI%": decodeURI,
        "%decodeURIComponent%": decodeURIComponent,
        "%encodeURI%": encodeURI,
        "%encodeURIComponent%": encodeURIComponent,
        "%Error%": Error,
        "%eval%": eval,
        "%EvalError%": EvalError,
        "%Float32Array%": typeof Float32Array == "undefined" ? c : Float32Array,
        "%Float64Array%": typeof Float64Array == "undefined" ? c : Float64Array,
        "%FinalizationRegistry%": typeof FinalizationRegistry == "undefined" ? c : FinalizationRegistry,
        "%Function%": ne,
        "%GeneratorFunction%": T,
        "%Int8Array%": typeof Int8Array == "undefined" ? c : Int8Array,
        "%Int16Array%": typeof Int16Array == "undefined" ? c : Int16Array,
        "%Int32Array%": typeof Int32Array == "undefined" ? c : Int32Array,
        "%isFinite%": isFinite,
        "%isNaN%": isNaN,
        "%IteratorPrototype%": E ? A(A([][Symbol.iterator]())) : c,
        "%JSON%": typeof JSON == "object" ? JSON : c,
        "%Map%": typeof Map == "undefined" ? c : Map,
        "%MapIteratorPrototype%": typeof Map == "undefined" || !E ? c : A(new Map()[Symbol.iterator]()),
        "%Math%": Math,
        "%Number%": Number,
        "%Object%": Object,
        "%parseFloat%": parseFloat,
        "%parseInt%": parseInt,
        "%Promise%": typeof Promise == "undefined" ? c : Promise,
        "%Proxy%": typeof Proxy == "undefined" ? c : Proxy,
        "%RangeError%": RangeError,
        "%ReferenceError%": ReferenceError,
        "%Reflect%": typeof Reflect == "undefined" ? c : Reflect,
        "%RegExp%": RegExp,
        "%Set%": typeof Set == "undefined" ? c : Set,
        "%SetIteratorPrototype%": typeof Set == "undefined" || !E ? c : A(new Set()[Symbol.iterator]()),
        "%SharedArrayBuffer%": typeof SharedArrayBuffer == "undefined" ? c : SharedArrayBuffer,
        "%String%": String,
        "%StringIteratorPrototype%": E ? A(""[Symbol.iterator]()) : c,
        "%Symbol%": E ? Symbol : c,
        "%SyntaxError%": B,
        "%ThrowTypeError%": Et,
        "%TypedArray%": Tt,
        "%TypeError%": F,
        "%Uint8Array%": typeof Uint8Array == "undefined" ? c : Uint8Array,
        "%Uint8ClampedArray%": typeof Uint8ClampedArray == "undefined" ? c : Uint8ClampedArray,
        "%Uint16Array%": typeof Uint16Array == "undefined" ? c : Uint16Array,
        "%Uint32Array%": typeof Uint32Array == "undefined" ? c : Uint32Array,
        "%URIError%": URIError,
        "%WeakMap%": typeof WeakMap == "undefined" ? c : WeakMap,
        "%WeakRef%": typeof WeakRef == "undefined" ? c : WeakRef,
        "%WeakSet%": typeof WeakSet == "undefined" ? c : WeakSet
    }, Ft = function r(e) {
        var t;
        if (e === "%AsyncFunction%") t = gr("async function () {}");
        else if (e === "%GeneratorFunction%") t = gr("function* () {}");
        else if (e === "%AsyncGeneratorFunction%") t = gr("async function* () {}");
        else if (e === "%AsyncGenerator%") {
            var n = r("%AsyncGeneratorFunction%");
            n && (t = n.prototype);
        } else if (e === "%AsyncIteratorPrototype%") {
            var o = r("%AsyncGenerator%");
            o && (t = A(o.prototype));
        }
        return I[e] = t, t;
    }, ee = {
        "%ArrayBufferPrototype%": [
            "ArrayBuffer",
            "prototype"
        ],
        "%ArrayPrototype%": [
            "Array",
            "prototype"
        ],
        "%ArrayProto_entries%": [
            "Array",
            "prototype",
            "entries"
        ],
        "%ArrayProto_forEach%": [
            "Array",
            "prototype",
            "forEach"
        ],
        "%ArrayProto_keys%": [
            "Array",
            "prototype",
            "keys"
        ],
        "%ArrayProto_values%": [
            "Array",
            "prototype",
            "values"
        ],
        "%AsyncFunctionPrototype%": [
            "AsyncFunction",
            "prototype"
        ],
        "%AsyncGenerator%": [
            "AsyncGeneratorFunction",
            "prototype"
        ],
        "%AsyncGeneratorPrototype%": [
            "AsyncGeneratorFunction",
            "prototype",
            "prototype"
        ],
        "%BooleanPrototype%": [
            "Boolean",
            "prototype"
        ],
        "%DataViewPrototype%": [
            "DataView",
            "prototype"
        ],
        "%DatePrototype%": [
            "Date",
            "prototype"
        ],
        "%ErrorPrototype%": [
            "Error",
            "prototype"
        ],
        "%EvalErrorPrototype%": [
            "EvalError",
            "prototype"
        ],
        "%Float32ArrayPrototype%": [
            "Float32Array",
            "prototype"
        ],
        "%Float64ArrayPrototype%": [
            "Float64Array",
            "prototype"
        ],
        "%FunctionPrototype%": [
            "Function",
            "prototype"
        ],
        "%Generator%": [
            "GeneratorFunction",
            "prototype"
        ],
        "%GeneratorPrototype%": [
            "GeneratorFunction",
            "prototype",
            "prototype"
        ],
        "%Int8ArrayPrototype%": [
            "Int8Array",
            "prototype"
        ],
        "%Int16ArrayPrototype%": [
            "Int16Array",
            "prototype"
        ],
        "%Int32ArrayPrototype%": [
            "Int32Array",
            "prototype"
        ],
        "%JSONParse%": [
            "JSON",
            "parse"
        ],
        "%JSONStringify%": [
            "JSON",
            "stringify"
        ],
        "%MapPrototype%": [
            "Map",
            "prototype"
        ],
        "%NumberPrototype%": [
            "Number",
            "prototype"
        ],
        "%ObjectPrototype%": [
            "Object",
            "prototype"
        ],
        "%ObjProto_toString%": [
            "Object",
            "prototype",
            "toString"
        ],
        "%ObjProto_valueOf%": [
            "Object",
            "prototype",
            "valueOf"
        ],
        "%PromisePrototype%": [
            "Promise",
            "prototype"
        ],
        "%PromiseProto_then%": [
            "Promise",
            "prototype",
            "then"
        ],
        "%Promise_all%": [
            "Promise",
            "all"
        ],
        "%Promise_reject%": [
            "Promise",
            "reject"
        ],
        "%Promise_resolve%": [
            "Promise",
            "resolve"
        ],
        "%RangeErrorPrototype%": [
            "RangeError",
            "prototype"
        ],
        "%ReferenceErrorPrototype%": [
            "ReferenceError",
            "prototype"
        ],
        "%RegExpPrototype%": [
            "RegExp",
            "prototype"
        ],
        "%SetPrototype%": [
            "Set",
            "prototype"
        ],
        "%SharedArrayBufferPrototype%": [
            "SharedArrayBuffer",
            "prototype"
        ],
        "%StringPrototype%": [
            "String",
            "prototype"
        ],
        "%SymbolPrototype%": [
            "Symbol",
            "prototype"
        ],
        "%SyntaxErrorPrototype%": [
            "SyntaxError",
            "prototype"
        ],
        "%TypedArrayPrototype%": [
            "TypedArray",
            "prototype"
        ],
        "%TypeErrorPrototype%": [
            "TypeError",
            "prototype"
        ],
        "%Uint8ArrayPrototype%": [
            "Uint8Array",
            "prototype"
        ],
        "%Uint8ClampedArrayPrototype%": [
            "Uint8ClampedArray",
            "prototype"
        ],
        "%Uint16ArrayPrototype%": [
            "Uint16Array",
            "prototype"
        ],
        "%Uint32ArrayPrototype%": [
            "Uint32Array",
            "prototype"
        ],
        "%URIErrorPrototype%": [
            "URIError",
            "prototype"
        ],
        "%WeakMapPrototype%": [
            "WeakMap",
            "prototype"
        ],
        "%WeakSetPrototype%": [
            "WeakSet",
            "prototype"
        ]
    }, M = W1(), z = re(), It = M.call(Function.call, Array.prototype.concat), Bt = M.call(Function.apply, Array.prototype.splice), te = M.call(Function.call, String.prototype.replace), V = M.call(Function.call, String.prototype.slice), Ut = M.call(Function.call, RegExp.prototype.exec), Rt = /[^%.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|%$))/g, Dt = /\\(\\)?/g, kt = function(e) {
        var t = V(e, 0, 1), n = V(e, -1);
        if (t === "%" && n !== "%") throw new B("invalid intrinsic syntax, expected closing `%`");
        if (n === "%" && t !== "%") throw new B("invalid intrinsic syntax, expected opening `%`");
        var o = [];
        return te(e, Rt, function(i, a, f, y) {
            o[o.length] = f ? te(y, Dt, "$1") : a || i;
        }), o;
    }, xt = function(e, t) {
        var n = e, o;
        if (z(ee, n) && (o = ee[n], n = "%" + o[0] + "%"), z(I, n)) {
            var i = I[n];
            if (i === T && (i = Ft(n)), typeof i == "undefined" && !t) throw new F("intrinsic " + e + " exists, but is not available. Please file an issue!");
            return {
                alias: o,
                name: n,
                value: i
            };
        }
        throw new B("intrinsic " + e + " does not exist!");
    };
    oe.exports = function(e, t) {
        if (typeof e != "string" || e.length === 0) throw new F("intrinsic name must be a non-empty string");
        if (arguments.length > 1 && typeof t != "boolean") throw new F('"allowMissing" argument must be a boolean');
        if (Ut(/^%?[^%]*%?$/, e) === null) throw new B("`%` may not be present anywhere but at the beginning and end of the intrinsic name");
        var n = kt(e), o = n.length > 0 ? n[0] : "", i = xt("%" + o + "%", t), a = i.name, f = i.value, y = !1, l = i.alias;
        l && (o = l[0], Bt(n, It([
            0,
            1
        ], l)));
        for(var g = 1, h = !0; g < n.length; g += 1){
            var d = n[g], w = V(d, 0, 1), G = V(d, -1);
            if ((w === '"' || w === "'" || w === "`" || G === '"' || G === "'" || G === "`") && w !== G) throw new B("property names with quotes must have matching quotes");
            if ((d === "constructor" || !h) && (y = !0), o += "." + d, a = "%" + o + "%", z(I, a)) f = I[a];
            else if (f != null) {
                if (!(d in f)) {
                    if (!t) throw new F("base intrinsic for " + e + " exists, but the property is not available.");
                    return;
                }
                if (v && g + 1 >= n.length) {
                    var _ = v(f, d);
                    h = !!_, h && "get" in _ && !("originalValue" in _.get) ? f = _.get : f = f[d];
                } else h = z(f, d), f = f[d];
                h && !y && (I[a] = f);
            }
        }
        return f;
    };
});
var ye = p((ho, L)=>{
    "use strict";
    var br = W1(), U = J(), fe = U("%Function.prototype.apply%"), se = U("%Function.prototype.call%"), ue = U("%Reflect.apply%", !0) || br.call(se, fe), ie = U("%Object.getOwnPropertyDescriptor%", !0), O = U("%Object.defineProperty%", !0), Mt = U("%Math.max%");
    if (O) try {
        O({}, "a", {
            value: 1
        });
    } catch (r) {
        O = null;
    }
    L.exports = function(e) {
        var t = ue(br, se, arguments);
        if (ie && O) {
            var n = ie(t, "length");
            n.configurable && O(t, "length", {
                value: 1 + Mt(0, e.length - (arguments.length - 1))
            });
        }
        return t;
    };
    var ae = function() {
        return ue(br, fe, arguments);
    };
    O ? O(L.exports, "apply", {
        value: ae
    }) : L.exports.apply = ae;
});
var H = p((vo, le)=>{
    "use strict";
    var ce = J(), pe = ye(), Nt = pe(ce("String.prototype.indexOf"));
    le.exports = function(e, t) {
        var n = ce(e, !!t);
        return typeof n == "function" && Nt(e, ".prototype.") > -1 ? pe(n) : n;
    };
});
var be = p((Oo, de)=>{
    "use strict";
    var Ct = x()(), $t = H(), mr = $t("Object.prototype.toString"), Z = function(e) {
        return Ct && e && typeof e == "object" && Symbol.toStringTag in e ? !1 : mr(e) === "[object Arguments]";
    }, ge = function(e) {
        return Z(e) ? !0 : e !== null && typeof e == "object" && typeof e.length == "number" && e.length >= 0 && mr(e) !== "[object Array]" && mr(e.callee) === "[object Function]";
    }, qt = function() {
        return Z(arguments);
    }();
    Z.isLegacyArguments = ge;
    de.exports = qt ? Z : ge;
});
var Se = p((jo, Ae)=>{
    "use strict";
    var Gt = Object.prototype.toString, _t = Function.prototype.toString, Wt = /^\s*(?:function)?\*/, me = x()(), Ar = Object.getPrototypeOf, zt = function() {
        if (!me) return !1;
        try {
            return Function("return function*() {}")();
        } catch (r) {}
    }, Sr;
    Ae.exports = function(e) {
        if (typeof e != "function") return !1;
        if (Wt.test(_t.call(e))) return !0;
        if (!me) {
            var t = Gt.call(e);
            return t === "[object GeneratorFunction]";
        }
        if (!Ar) return !1;
        if (typeof Sr == "undefined") {
            var n = zt();
            Sr = n ? Ar(n) : !1;
        }
        return Ar(e) === Sr;
    };
});
var je = p((Po, Oe)=>{
    "use strict";
    var ve = Function.prototype.toString, R = typeof Reflect == "object" && Reflect !== null && Reflect.apply, vr, Y;
    if (typeof R == "function" && typeof Object.defineProperty == "function") try {
        vr = Object.defineProperty({}, "length", {
            get: function() {
                throw Y;
            }
        }), Y = {}, R(function() {
            throw 42;
        }, null, vr);
    } catch (r) {
        r !== Y && (R = null);
    }
    else R = null;
    var Vt = /^\s*class\b/, Or = function(e) {
        try {
            var t = ve.call(e);
            return Vt.test(t);
        } catch (n) {
            return !1;
        }
    }, hr = function(e) {
        try {
            return Or(e) ? !1 : (ve.call(e), !0);
        } catch (t) {
            return !1;
        }
    }, K = Object.prototype.toString, Jt = "[object Object]", Lt = "[object Function]", Ht = "[object GeneratorFunction]", Zt = "[object HTMLAllCollection]", Yt = "[object HTML document.all class]", Kt = "[object HTMLCollection]", Qt = typeof Symbol == "function" && !!Symbol.toStringTag, Xt = !(0 in [
        ,
    ]), jr = function() {
        return !1;
    };
    typeof document == "object" && (he = document.all, K.call(he) === K.call(document.all) && (jr = function(e) {
        if ((Xt || !e) && (typeof e == "undefined" || typeof e == "object")) try {
            var t = K.call(e);
            return (t === Zt || t === Yt || t === Kt || t === Jt) && e("") == null;
        } catch (n) {}
        return !1;
    }));
    var he;
    Oe.exports = R ? function(e) {
        if (jr(e)) return !0;
        if (!e || typeof e != "function" && typeof e != "object") return !1;
        try {
            R(e, null, vr);
        } catch (t) {
            if (t !== Y) return !1;
        }
        return !Or(e) && hr(e);
    } : function(e) {
        if (jr(e)) return !0;
        if (!e || typeof e != "function" && typeof e != "object") return !1;
        if (Qt) return hr(e);
        if (Or(e)) return !1;
        var t = K.call(e);
        return t !== Lt && t !== Ht && !/^\[object HTML/.test(t) ? !1 : hr(e);
    };
});
var Pr = p((wo, we)=>{
    "use strict";
    var rn = je(), en = Object.prototype.toString, Pe = Object.prototype.hasOwnProperty, tn = function(e, t, n) {
        for(var o = 0, i = e.length; o < i; o++)Pe.call(e, o) && (n == null ? t(e[o], o, e) : t.call(n, e[o], o, e));
    }, nn = function(e, t, n) {
        for(var o = 0, i = e.length; o < i; o++)n == null ? t(e.charAt(o), o, e) : t.call(n, e.charAt(o), o, e);
    }, on = function(e, t, n) {
        for(var o in e)Pe.call(e, o) && (n == null ? t(e[o], o, e) : t.call(n, e[o], o, e));
    }, an = function(e, t, n) {
        if (!rn(t)) throw new TypeError("iterator must be a function");
        var o;
        arguments.length >= 3 && (o = n), en.call(e) === "[object Array]" ? tn(e, t, o) : typeof e == "string" ? nn(e, t, o) : on(e, t, o);
    };
    we.exports = an;
});
var Er = p((Eo, Ee)=>{
    "use strict";
    var wr = [
        "BigInt64Array",
        "BigUint64Array",
        "Float32Array",
        "Float64Array",
        "Int16Array",
        "Int32Array",
        "Int8Array",
        "Uint16Array",
        "Uint32Array",
        "Uint8Array",
        "Uint8ClampedArray"
    ], fn = typeof globalThis == "undefined" ? __global$ : globalThis;
    Ee.exports = function() {
        for(var e = [], t = 0; t < wr.length; t++)typeof fn[wr[t]] == "function" && (e[e.length] = wr[t]);
        return e;
    };
});
var Tr = p((To, Te)=>{
    "use strict";
    var sn = J(), Q = sn("%Object.getOwnPropertyDescriptor%", !0);
    if (Q) try {
        Q([], "length");
    } catch (r) {
        Q = null;
    }
    Te.exports = Q;
});
var Br = p((Fo, Re)=>{
    "use strict";
    var Fe = Pr(), un = Er(), Ir = H(), yn = Ir("Object.prototype.toString"), Ie = x()(), X = Tr(), cn = typeof globalThis == "undefined" ? __global$ : globalThis, Be = un(), pn = Ir("Array.prototype.indexOf", !0) || function(e, t) {
        for(var n = 0; n < e.length; n += 1)if (e[n] === t) return n;
        return -1;
    }, ln = Ir("String.prototype.slice"), Ue = {}, Fr = Object.getPrototypeOf;
    Ie && X && Fr && Fe(Be, function(r) {
        var e = new cn[r];
        if (Symbol.toStringTag in e) {
            var t = Fr(e), n = X(t, Symbol.toStringTag);
            if (!n) {
                var o = Fr(t);
                n = X(o, Symbol.toStringTag);
            }
            Ue[r] = n.get;
        }
    });
    var gn = function(e) {
        var t = !1;
        return Fe(Ue, function(n, o) {
            if (!t) try {
                t = n.call(e) === o;
            } catch (i) {}
        }), t;
    };
    Re.exports = function(e) {
        if (!e || typeof e != "object") return !1;
        if (!Ie || !(Symbol.toStringTag in e)) {
            var t = ln(yn(e), 8, -1);
            return pn(Be, t) > -1;
        }
        return X ? gn(e) : !1;
    };
});
var $e = p((Io, Ce)=>{
    "use strict";
    var ke = Pr(), dn = Er(), xe = H(), Ur = Tr(), bn = xe("Object.prototype.toString"), Me = x()(), De = typeof globalThis == "undefined" ? __global$ : globalThis, mn = dn(), An = xe("String.prototype.slice"), Ne = {}, Rr = Object.getPrototypeOf;
    Me && Ur && Rr && ke(mn, function(r) {
        if (typeof De[r] == "function") {
            var e = new De[r];
            if (Symbol.toStringTag in e) {
                var t = Rr(e), n = Ur(t, Symbol.toStringTag);
                if (!n) {
                    var o = Rr(t);
                    n = Ur(o, Symbol.toStringTag);
                }
                Ne[r] = n.get;
            }
        }
    });
    var Sn = function(e) {
        var t = !1;
        return ke(Ne, function(n, o) {
            if (!t) try {
                var i = n.call(e);
                i === o && (t = i);
            } catch (a) {}
        }), t;
    }, hn = Br();
    Ce.exports = function(e) {
        return hn(e) ? !Me || !(Symbol.toStringTag in e) ? An(bn(e), 8, -1) : Sn(e) : !1;
    };
});
var Xe = p((s)=>{
    "use strict";
    var vn = be(), On = Se(), m = $e(), qe = Br();
    function D(r) {
        return r.call.bind(r);
    }
    var Ge = typeof BigInt != "undefined", _e = typeof Symbol != "undefined", b = D(Object.prototype.toString), jn = D(Number.prototype.valueOf), Pn = D(String.prototype.valueOf), wn = D(Boolean.prototype.valueOf);
    Ge && (We = D(BigInt.prototype.valueOf));
    var We;
    _e && (ze = D(Symbol.prototype.valueOf));
    var ze;
    function C(r, e) {
        if (typeof r != "object") return !1;
        try {
            return e(r), !0;
        } catch (t) {
            return !1;
        }
    }
    s.isArgumentsObject = vn;
    s.isGeneratorFunction = On;
    s.isTypedArray = qe;
    function En(r) {
        return typeof Promise != "undefined" && r instanceof Promise || r !== null && typeof r == "object" && typeof r.then == "function" && typeof r.catch == "function";
    }
    s.isPromise = En;
    function Tn(r) {
        return typeof ArrayBuffer != "undefined" && ArrayBuffer.isView ? ArrayBuffer.isView(r) : qe(r) || Je(r);
    }
    s.isArrayBufferView = Tn;
    function Fn(r) {
        return m(r) === "Uint8Array";
    }
    s.isUint8Array = Fn;
    function In(r) {
        return m(r) === "Uint8ClampedArray";
    }
    s.isUint8ClampedArray = In;
    function Bn(r) {
        return m(r) === "Uint16Array";
    }
    s.isUint16Array = Bn;
    function Un(r) {
        return m(r) === "Uint32Array";
    }
    s.isUint32Array = Un;
    function Rn(r) {
        return m(r) === "Int8Array";
    }
    s.isInt8Array = Rn;
    function Dn(r) {
        return m(r) === "Int16Array";
    }
    s.isInt16Array = Dn;
    function kn(r) {
        return m(r) === "Int32Array";
    }
    s.isInt32Array = kn;
    function xn(r) {
        return m(r) === "Float32Array";
    }
    s.isFloat32Array = xn;
    function Mn(r) {
        return m(r) === "Float64Array";
    }
    s.isFloat64Array = Mn;
    function Nn(r) {
        return m(r) === "BigInt64Array";
    }
    s.isBigInt64Array = Nn;
    function Cn(r) {
        return m(r) === "BigUint64Array";
    }
    s.isBigUint64Array = Cn;
    function rr(r) {
        return b(r) === "[object Map]";
    }
    rr.working = typeof Map != "undefined" && rr(new Map);
    function $n(r) {
        return typeof Map == "undefined" ? !1 : rr.working ? rr(r) : r instanceof Map;
    }
    s.isMap = $n;
    function er(r) {
        return b(r) === "[object Set]";
    }
    er.working = typeof Set != "undefined" && er(new Set);
    function qn(r) {
        return typeof Set == "undefined" ? !1 : er.working ? er(r) : r instanceof Set;
    }
    s.isSet = qn;
    function tr(r) {
        return b(r) === "[object WeakMap]";
    }
    tr.working = typeof WeakMap != "undefined" && tr(new WeakMap);
    function Gn(r) {
        return typeof WeakMap == "undefined" ? !1 : tr.working ? tr(r) : r instanceof WeakMap;
    }
    s.isWeakMap = Gn;
    function kr(r) {
        return b(r) === "[object WeakSet]";
    }
    kr.working = typeof WeakSet != "undefined" && kr(new WeakSet);
    function _n(r) {
        return kr(r);
    }
    s.isWeakSet = _n;
    function nr(r) {
        return b(r) === "[object ArrayBuffer]";
    }
    nr.working = typeof ArrayBuffer != "undefined" && nr(new ArrayBuffer);
    function Ve(r) {
        return typeof ArrayBuffer == "undefined" ? !1 : nr.working ? nr(r) : r instanceof ArrayBuffer;
    }
    s.isArrayBuffer = Ve;
    function or(r) {
        return b(r) === "[object DataView]";
    }
    or.working = typeof ArrayBuffer != "undefined" && typeof DataView != "undefined" && or(new DataView(new ArrayBuffer(1), 0, 1));
    function Je(r) {
        return typeof DataView == "undefined" ? !1 : or.working ? or(r) : r instanceof DataView;
    }
    s.isDataView = Je;
    var Dr = typeof SharedArrayBuffer != "undefined" ? SharedArrayBuffer : void 0;
    function N(r) {
        return b(r) === "[object SharedArrayBuffer]";
    }
    function Le(r) {
        return typeof Dr == "undefined" ? !1 : (typeof N.working == "undefined" && (N.working = N(new Dr)), N.working ? N(r) : r instanceof Dr);
    }
    s.isSharedArrayBuffer = Le;
    function Wn(r) {
        return b(r) === "[object AsyncFunction]";
    }
    s.isAsyncFunction = Wn;
    function zn(r) {
        return b(r) === "[object Map Iterator]";
    }
    s.isMapIterator = zn;
    function Vn(r) {
        return b(r) === "[object Set Iterator]";
    }
    s.isSetIterator = Vn;
    function Jn(r) {
        return b(r) === "[object Generator]";
    }
    s.isGeneratorObject = Jn;
    function Ln(r) {
        return b(r) === "[object WebAssembly.Module]";
    }
    s.isWebAssemblyCompiledModule = Ln;
    function He(r) {
        return C(r, jn);
    }
    s.isNumberObject = He;
    function Ze(r) {
        return C(r, Pn);
    }
    s.isStringObject = Ze;
    function Ye(r) {
        return C(r, wn);
    }
    s.isBooleanObject = Ye;
    function Ke(r) {
        return Ge && C(r, We);
    }
    s.isBigIntObject = Ke;
    function Qe(r) {
        return _e && C(r, ze);
    }
    s.isSymbolObject = Qe;
    function Hn(r) {
        return He(r) || Ze(r) || Ye(r) || Ke(r) || Qe(r);
    }
    s.isBoxedPrimitive = Hn;
    function Zn(r) {
        return typeof Uint8Array != "undefined" && (Ve(r) || Le(r));
    }
    s.isAnyArrayBuffer = Zn;
    [
        "isProxy",
        "isExternal",
        "isModuleNamespaceObject"
    ].forEach(function(r) {
        Object.defineProperty(s, r, {
            enumerable: !1,
            value: function() {
                throw new Error(r + " is not supported in userland");
            }
        });
    });
});
var et = p((Uo, rt)=>{
    rt.exports = function(e) {
        return e && typeof e == "object" && typeof e.copy == "function" && typeof e.fill == "function" && typeof e.readUInt8 == "function";
    };
});
var tt = p((Ro, xr)=>{
    typeof Object.create == "function" ? xr.exports = function(e, t) {
        t && (e.super_ = t, e.prototype = Object.create(t.prototype, {
            constructor: {
                value: e,
                enumerable: !1,
                writable: !0,
                configurable: !0
            }
        }));
    } : xr.exports = function(e, t) {
        if (t) {
            e.super_ = t;
            var n = function() {};
            n.prototype = t.prototype, e.prototype = new n, e.prototype.constructor = e;
        }
    };
});
var st = p((u)=>{
    var nt = Object.getOwnPropertyDescriptors || function(e) {
        for(var t = Object.keys(e), n = {}, o = 0; o < t.length; o++)n[t[o]] = Object.getOwnPropertyDescriptor(e, t[o]);
        return n;
    }, Yn = /%[sdj%]/g;
    u.format = function(r) {
        if (!cr(r)) {
            for(var e = [], t = 0; t < arguments.length; t++)e.push(S(arguments[t]));
            return e.join(" ");
        }
        for(var t = 1, n = arguments, o = n.length, i = String(r).replace(Yn, function(f) {
            if (f === "%%") return "%";
            if (t >= o) return f;
            switch(f){
                case "%s":
                    return String(n[t++]);
                case "%d":
                    return Number(n[t++]);
                case "%j":
                    try {
                        return JSON.stringify(n[t++]);
                    } catch (y) {
                        return "[Circular]";
                    }
                default:
                    return f;
            }
        }), a = n[t]; t < o; a = n[++t])yr(a) || !k(a) ? i += " " + a : i += " " + S(a);
        return i;
    };
    u.deprecate = function(r, e) {
        if (typeof __default != "undefined" && __default.noDeprecation === !0) return r;
        if (typeof __default == "undefined") return function() {
            return u.deprecate(r, e).apply(this, arguments);
        };
        var t = !1;
        function n() {
            if (!t) {
                if (__default.throwDeprecation) throw new Error(e);
                __default.traceDeprecation ? console.trace(e) : console.error(e), t = !0;
            }
            return r.apply(this, arguments);
        }
        return n;
    };
    var ir = {}, ot = /^$/;
    __default.env.NODE_DEBUG && (ar = __default.env.NODE_DEBUG, ar = ar.replace(/[|\\{}()[\]^$+?.]/g, "\\$&").replace(/\*/g, ".*").replace(/,/g, "$|^").toUpperCase(), ot = new RegExp("^" + ar + "$", "i"));
    var ar;
    u.debuglog = function(r) {
        if (r = r.toUpperCase(), !ir[r]) if (ot.test(r)) {
            var e = __default.pid;
            ir[r] = function() {
                var t = u.format.apply(u, arguments);
                console.error("%s %d: %s", r, e, t);
            };
        } else ir[r] = function() {};
        return ir[r];
    };
    function S(r, e) {
        var t = {
            seen: [],
            stylize: Qn
        };
        return arguments.length >= 3 && (t.depth = arguments[2]), arguments.length >= 4 && (t.colors = arguments[3]), $r(e) ? t.showHidden = e : e && u._extend(t, e), P(t.showHidden) && (t.showHidden = !1), P(t.depth) && (t.depth = 2), P(t.colors) && (t.colors = !1), P(t.customInspect) && (t.customInspect = !0), t.colors && (t.stylize = Kn), sr(t, r, t.depth);
    }
    u.inspect = S;
    S.colors = {
        bold: [
            1,
            22
        ],
        italic: [
            3,
            23
        ],
        underline: [
            4,
            24
        ],
        inverse: [
            7,
            27
        ],
        white: [
            37,
            39
        ],
        grey: [
            90,
            39
        ],
        black: [
            30,
            39
        ],
        blue: [
            34,
            39
        ],
        cyan: [
            36,
            39
        ],
        green: [
            32,
            39
        ],
        magenta: [
            35,
            39
        ],
        red: [
            31,
            39
        ],
        yellow: [
            33,
            39
        ]
    };
    S.styles = {
        special: "cyan",
        number: "yellow",
        boolean: "yellow",
        undefined: "grey",
        null: "bold",
        string: "green",
        date: "magenta",
        regexp: "red"
    };
    function Kn(r, e) {
        var t = S.styles[e];
        return t ? "\x1B[" + S.colors[t][0] + "m" + r + "\x1B[" + S.colors[t][1] + "m" : r;
    }
    function Qn(r, e) {
        return r;
    }
    function Xn(r) {
        var e = {};
        return r.forEach(function(t, n) {
            e[t] = !0;
        }), e;
    }
    function sr(r, e, t) {
        if (r.customInspect && e && fr(e.inspect) && e.inspect !== u.inspect && !(e.constructor && e.constructor.prototype === e)) {
            var n = e.inspect(t, r);
            return cr(n) || (n = sr(r, n, t)), n;
        }
        var o = ro(r, e);
        if (o) return o;
        var i = Object.keys(e), a = Xn(i);
        if (r.showHidden && (i = Object.getOwnPropertyNames(e)), q(e) && (i.indexOf("message") >= 0 || i.indexOf("description") >= 0)) return Mr(e);
        if (i.length === 0) {
            if (fr(e)) {
                var f = e.name ? ": " + e.name : "";
                return r.stylize("[Function" + f + "]", "special");
            }
            if ($(e)) return r.stylize(RegExp.prototype.toString.call(e), "regexp");
            if (ur(e)) return r.stylize(Date.prototype.toString.call(e), "date");
            if (q(e)) return Mr(e);
        }
        var y = "", l = !1, g = [
            "{",
            "}"
        ];
        if (it(e) && (l = !0, g = [
            "[",
            "]"
        ]), fr(e)) {
            var h = e.name ? ": " + e.name : "";
            y = " [Function" + h + "]";
        }
        if ($(e) && (y = " " + RegExp.prototype.toString.call(e)), ur(e) && (y = " " + Date.prototype.toUTCString.call(e)), q(e) && (y = " " + Mr(e)), i.length === 0 && (!l || e.length == 0)) return g[0] + y + g[1];
        if (t < 0) return $(e) ? r.stylize(RegExp.prototype.toString.call(e), "regexp") : r.stylize("[Object]", "special");
        r.seen.push(e);
        var d;
        return l ? d = eo(r, e, t, a, i) : d = i.map(function(w) {
            return Cr(r, e, t, a, w, l);
        }), r.seen.pop(), to(d, y, g);
    }
    function ro(r, e) {
        if (P(e)) return r.stylize("undefined", "undefined");
        if (cr(e)) {
            var t = "'" + JSON.stringify(e).replace(/^"|"$/g, "").replace(/'/g, "\\'").replace(/\\"/g, '"') + "'";
            return r.stylize(t, "string");
        }
        if (at(e)) return r.stylize("" + e, "number");
        if ($r(e)) return r.stylize("" + e, "boolean");
        if (yr(e)) return r.stylize("null", "null");
    }
    function Mr(r) {
        return "[" + Error.prototype.toString.call(r) + "]";
    }
    function eo(r, e, t, n, o) {
        for(var i = [], a = 0, f = e.length; a < f; ++a)ft(e, String(a)) ? i.push(Cr(r, e, t, n, String(a), !0)) : i.push("");
        return o.forEach(function(y) {
            y.match(/^\d+$/) || i.push(Cr(r, e, t, n, y, !0));
        }), i;
    }
    function Cr(r, e, t, n, o, i) {
        var a, f, y;
        if (y = Object.getOwnPropertyDescriptor(e, o) || {
            value: e[o]
        }, y.get ? y.set ? f = r.stylize("[Getter/Setter]", "special") : f = r.stylize("[Getter]", "special") : y.set && (f = r.stylize("[Setter]", "special")), ft(n, o) || (a = "[" + o + "]"), f || (r.seen.indexOf(y.value) < 0 ? (yr(t) ? f = sr(r, y.value, null) : f = sr(r, y.value, t - 1), f.indexOf(`
`) > -1 && (i ? f = f.split(`
`).map(function(l) {
            return "  " + l;
        }).join(`
`).slice(2) : f = `
` + f.split(`
`).map(function(l) {
            return "   " + l;
        }).join(`
`))) : f = r.stylize("[Circular]", "special")), P(a)) {
            if (i && o.match(/^\d+$/)) return f;
            a = JSON.stringify("" + o), a.match(/^"([a-zA-Z_][a-zA-Z_0-9]*)"$/) ? (a = a.slice(1, -1), a = r.stylize(a, "name")) : (a = a.replace(/'/g, "\\'").replace(/\\"/g, '"').replace(/(^"|"$)/g, "'"), a = r.stylize(a, "string"));
        }
        return a + ": " + f;
    }
    function to(r, e, t) {
        var n = 0, o = r.reduce(function(i, a) {
            return n++, a.indexOf(`
`) >= 0 && n++, i + a.replace(/\u001b\[\d\d?m/g, "").length + 1;
        }, 0);
        return o > 60 ? t[0] + (e === "" ? "" : e + `
 `) + " " + r.join(`,
  `) + " " + t[1] : t[0] + e + " " + r.join(", ") + " " + t[1];
    }
    u.types = Xe();
    function it(r) {
        return Array.isArray(r);
    }
    u.isArray = it;
    function $r(r) {
        return typeof r == "boolean";
    }
    u.isBoolean = $r;
    function yr(r) {
        return r === null;
    }
    u.isNull = yr;
    function no(r) {
        return r == null;
    }
    u.isNullOrUndefined = no;
    function at(r) {
        return typeof r == "number";
    }
    u.isNumber = at;
    function cr(r) {
        return typeof r == "string";
    }
    u.isString = cr;
    function oo(r) {
        return typeof r == "symbol";
    }
    u.isSymbol = oo;
    function P(r) {
        return r === void 0;
    }
    u.isUndefined = P;
    function $(r) {
        return k(r) && qr(r) === "[object RegExp]";
    }
    u.isRegExp = $;
    u.types.isRegExp = $;
    function k(r) {
        return typeof r == "object" && r !== null;
    }
    u.isObject = k;
    function ur(r) {
        return k(r) && qr(r) === "[object Date]";
    }
    u.isDate = ur;
    u.types.isDate = ur;
    function q(r) {
        return k(r) && (qr(r) === "[object Error]" || r instanceof Error);
    }
    u.isError = q;
    u.types.isNativeError = q;
    function fr(r) {
        return typeof r == "function";
    }
    u.isFunction = fr;
    function io(r) {
        return r === null || typeof r == "boolean" || typeof r == "number" || typeof r == "string" || typeof r == "symbol" || typeof r == "undefined";
    }
    u.isPrimitive = io;
    u.isBuffer = et();
    function qr(r) {
        return Object.prototype.toString.call(r);
    }
    function Nr(r) {
        return r < 10 ? "0" + r.toString(10) : r.toString(10);
    }
    var ao = [
        "Jan",
        "Feb",
        "Mar",
        "Apr",
        "May",
        "Jun",
        "Jul",
        "Aug",
        "Sep",
        "Oct",
        "Nov",
        "Dec"
    ];
    function fo() {
        var r = new Date, e = [
            Nr(r.getHours()),
            Nr(r.getMinutes()),
            Nr(r.getSeconds())
        ].join(":");
        return [
            r.getDate(),
            ao[r.getMonth()],
            e
        ].join(" ");
    }
    u.log = function() {
        console.log("%s - %s", fo(), u.format.apply(u, arguments));
    };
    u.inherits = tt();
    u._extend = function(r, e) {
        if (!e || !k(e)) return r;
        for(var t = Object.keys(e), n = t.length; n--;)r[t[n]] = e[t[n]];
        return r;
    };
    function ft(r, e) {
        return Object.prototype.hasOwnProperty.call(r, e);
    }
    var j = typeof Symbol != "undefined" ? Symbol("util.promisify.custom") : void 0;
    u.promisify = function(e) {
        if (typeof e != "function") throw new TypeError('The "original" argument must be of type Function');
        if (j && e[j]) {
            var t = e[j];
            if (typeof t != "function") throw new TypeError('The "util.promisify.custom" argument must be of type Function');
            return Object.defineProperty(t, j, {
                value: t,
                enumerable: !1,
                writable: !1,
                configurable: !0
            }), t;
        }
        function t() {
            for(var n, o, i = new Promise(function(y, l) {
                n = y, o = l;
            }), a = [], f = 0; f < arguments.length; f++)a.push(arguments[f]);
            a.push(function(y, l) {
                y ? o(y) : n(l);
            });
            try {
                e.apply(this, a);
            } catch (y) {
                o(y);
            }
            return i;
        }
        return Object.setPrototypeOf(t, Object.getPrototypeOf(e)), j && Object.defineProperty(t, j, {
            value: t,
            enumerable: !1,
            writable: !1,
            configurable: !0
        }), Object.defineProperties(t, nt(e));
    };
    u.promisify.custom = j;
    function so(r, e) {
        if (!r) {
            var t = new Error("Promise was rejected with a falsy value");
            t.reason = r, r = t;
        }
        return e(r);
    }
    function uo(r) {
        if (typeof r != "function") throw new TypeError('The "original" argument must be of type Function');
        function e() {
            for(var t = [], n = 0; n < arguments.length; n++)t.push(arguments[n]);
            var o = t.pop();
            if (typeof o != "function") throw new TypeError("The last argument must be of type Function");
            var i = this, a = function() {
                return o.apply(i, arguments);
            };
            r.apply(this, t).then(function(f) {
                __default.nextTick(a.bind(null, null, f));
            }, function(f) {
                __default.nextTick(so.bind(null, f, a));
            });
        }
        return Object.setPrototypeOf(e, Object.getPrototypeOf(r)), Object.defineProperties(e, nt(r)), e;
    }
    u.callbackify = uo;
});
var ct = At(st());
var { format: ko , deprecate: xo , debuglog: Mo , inspect: No , types: Co , isArray: $o , isBoolean: qo , isNull: Go , isNullOrUndefined: _o , isNumber: Wo , isString: zo , isSymbol: Vo , isUndefined: Jo , isRegExp: Lo , isObject: Ho , isDate: Zo , isError: Yo , isFunction: Ko , isPrimitive: Qo , isBuffer: Xo , log: ri , inherits: ei , _extend: ti , promisify: ni , callbackify: oi  } = ct, ut = ct, { default: yt  } = ut, yo = zr(ut, [
    "default"
]), ii = yt !== void 0 ? yt : yo;
var fn = Object.create;
var ut1 = Object.defineProperty;
var sn = Object.getOwnPropertyDescriptor;
var ln = Object.getOwnPropertyNames, ct1 = Object.getOwnPropertySymbols, pn = Object.getPrototypeOf, ft = Object.prototype.hasOwnProperty, yn = Object.prototype.propertyIsEnumerable;
((t)=>typeof require != "undefined" ? require : typeof Proxy != "undefined" ? new Proxy(t, {
        get: (e, r)=>(typeof require != "undefined" ? require : e)[r]
    }) : t)(function(t) {
    if (typeof require != "undefined") return require.apply(this, arguments);
    throw new Error('Dynamic require of "' + t + '" is not supported');
});
var st1 = (t, e)=>{
    var r = {};
    for(var n in t)ft.call(t, n) && e.indexOf(n) < 0 && (r[n] = t[n]);
    if (t != null && ct1) for (var n of ct1(t))e.indexOf(n) < 0 && yn.call(t, n) && (r[n] = t[n]);
    return r;
};
var g = (t, e)=>()=>(e || t((e = {
            exports: {}
        }).exports, e), e.exports);
var gn = (t, e, r, n)=>{
    if (e && typeof e == "object" || typeof e == "function") for (let o of ln(e))!ft.call(t, o) && o !== r && ut1(t, o, {
        get: ()=>e[o],
        enumerable: !(n = sn(e, o)) || n.enumerable
    });
    return t;
};
var hn = (t, e, r)=>(r = t != null ? fn(pn(t)) : {}, gn(e || !t || !t.__esModule ? ut1(r, "default", {
        value: t,
        enumerable: !0
    }) : r, t));
var Fe = g((fi, yt)=>{
    "use strict";
    function $(t) {
        return typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? $ = function(r) {
            return typeof r;
        } : $ = function(r) {
            return r && typeof Symbol == "function" && r.constructor === Symbol && r !== Symbol.prototype ? "symbol" : typeof r;
        }, $(t);
    }
    function dn(t, e) {
        if (!(t instanceof e)) throw new TypeError("Cannot call a class as a function");
    }
    function bn(t, e) {
        return e && ($(e) === "object" || typeof e == "function") ? e : vn(t);
    }
    function vn(t) {
        if (t === void 0) throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
        return t;
    }
    function Ne(t) {
        return Ne = Object.setPrototypeOf ? Object.getPrototypeOf : function(r) {
            return r.__proto__ || Object.getPrototypeOf(r);
        }, Ne(t);
    }
    function mn(t, e) {
        if (typeof e != "function" && e !== null) throw new TypeError("Super expression must either be null or a function");
        t.prototype = Object.create(e && e.prototype, {
            constructor: {
                value: t,
                writable: !0,
                configurable: !0
            }
        }), e && xe(t, e);
    }
    function xe(t, e) {
        return xe = Object.setPrototypeOf || function(n, o) {
            return n.__proto__ = o, n;
        }, xe(t, e);
    }
    var pt = {}, G, Ie;
    function X(t, e, r) {
        r || (r = Error);
        function n(i, a, c) {
            return typeof e == "string" ? e : e(i, a, c);
        }
        var o = function(i) {
            mn(a, i);
            function a(c, u, f) {
                var l;
                return dn(this, a), l = bn(this, Ne(a).call(this, n(c, u, f))), l.code = t, l;
            }
            return a;
        }(r);
        pt[t] = o;
    }
    function lt(t, e) {
        if (Array.isArray(t)) {
            var r = t.length;
            return t = t.map(function(n) {
                return String(n);
            }), r > 2 ? "one of ".concat(e, " ").concat(t.slice(0, r - 1).join(", "), ", or ") + t[r - 1] : r === 2 ? "one of ".concat(e, " ").concat(t[0], " or ").concat(t[1]) : "of ".concat(e, " ").concat(t[0]);
        } else return "of ".concat(e, " ").concat(String(t));
    }
    function En(t, e, r) {
        return t.substr(!r || r < 0 ? 0 : +r, e.length) === e;
    }
    function Sn(t, e, r) {
        return (r === void 0 || r > t.length) && (r = t.length), t.substring(r - e.length, r) === e;
    }
    function On(t, e, r) {
        return typeof r != "number" && (r = 0), r + e.length > t.length ? !1 : t.indexOf(e, r) !== -1;
    }
    X("ERR_AMBIGUOUS_ARGUMENT", 'The "%s" argument is ambiguous. %s', TypeError);
    X("ERR_INVALID_ARG_TYPE", function(t, e, r) {
        G === void 0 && (G = se()), G(typeof t == "string", "'name' must be a string");
        var n;
        typeof e == "string" && En(e, "not ") ? (n = "must not be", e = e.replace(/^not /, "")) : n = "must be";
        var o;
        if (Sn(t, " argument")) o = "The ".concat(t, " ").concat(n, " ").concat(lt(e, "type"));
        else {
            var i = On(t, ".") ? "property" : "argument";
            o = 'The "'.concat(t, '" ').concat(i, " ").concat(n, " ").concat(lt(e, "type"));
        }
        return o += ". Received type ".concat($(r)), o;
    }, TypeError);
    X("ERR_INVALID_ARG_VALUE", function(t, e) {
        var r = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : "is invalid";
        Ie === void 0 && (Ie = ii);
        var n = Ie.inspect(e);
        return n.length > 128 && (n = "".concat(n.slice(0, 128), "...")), "The argument '".concat(t, "' ").concat(r, ". Received ").concat(n);
    }, TypeError, RangeError);
    X("ERR_INVALID_RETURN_VALUE", function(t, e, r) {
        var n;
        return r && r.constructor && r.constructor.name ? n = "instance of ".concat(r.constructor.name) : n = "type ".concat($(r)), "Expected ".concat(t, ' to be returned from the "').concat(e, '"') + " function but got ".concat(n, ".");
    }, TypeError);
    X("ERR_MISSING_ARGS", function() {
        for(var t = arguments.length, e = new Array(t), r = 0; r < t; r++)e[r] = arguments[r];
        G === void 0 && (G = se()), G(e.length > 0, "At least one arg needs to be specified");
        var n = "The ", o = e.length;
        switch(e = e.map(function(i) {
            return '"'.concat(i, '"');
        }), o){
            case 1:
                n += "".concat(e[0], " argument");
                break;
            case 2:
                n += "".concat(e[0], " and ").concat(e[1], " arguments");
                break;
            default:
                n += e.slice(0, o - 1).join(", "), n += ", and ".concat(e[o - 1], " arguments");
                break;
        }
        return "".concat(n, " must be specified");
    }, TypeError);
    yt.exports.codes = pt;
});
var vt = g((si, bt)=>{
    "use strict";
    function wn(t) {
        for(var e = 1; e < arguments.length; e++){
            var r = arguments[e] != null ? arguments[e] : {}, n = Object.keys(r);
            typeof Object.getOwnPropertySymbols == "function" && (n = n.concat(Object.getOwnPropertySymbols(r).filter(function(o) {
                return Object.getOwnPropertyDescriptor(r, o).enumerable;
            }))), n.forEach(function(o) {
                An(t, o, r[o]);
            });
        }
        return t;
    }
    function An(t, e, r) {
        return e in t ? Object.defineProperty(t, e, {
            value: r,
            enumerable: !0,
            configurable: !0,
            writable: !0
        }) : t[e] = r, t;
    }
    function Pn(t, e) {
        if (!(t instanceof e)) throw new TypeError("Cannot call a class as a function");
    }
    function gt(t, e) {
        for(var r = 0; r < e.length; r++){
            var n = e[r];
            n.enumerable = n.enumerable || !1, n.configurable = !0, "value" in n && (n.writable = !0), Object.defineProperty(t, n.key, n);
        }
    }
    function jn(t, e, r) {
        return e && gt(t.prototype, e), r && gt(t, r), t;
    }
    function B(t, e) {
        return e && (S(e) === "object" || typeof e == "function") ? e : Te(t);
    }
    function Te(t) {
        if (t === void 0) throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
        return t;
    }
    function qn(t, e) {
        if (typeof e != "function" && e !== null) throw new TypeError("Super expression must either be null or a function");
        t.prototype = Object.create(e && e.prototype, {
            constructor: {
                value: t,
                writable: !0,
                configurable: !0
            }
        }), e && ee(t, e);
    }
    function _e(t) {
        var e = typeof Map == "function" ? new Map : void 0;
        return _e = function(n) {
            if (n === null || !In(n)) return n;
            if (typeof n != "function") throw new TypeError("Super expression must either be null or a function");
            if (typeof e != "undefined") {
                if (e.has(n)) return e.get(n);
                e.set(n, o);
            }
            function o() {
                return le(n, arguments, R(this).constructor);
            }
            return o.prototype = Object.create(n.prototype, {
                constructor: {
                    value: o,
                    enumerable: !1,
                    writable: !0,
                    configurable: !0
                }
            }), ee(o, n);
        }, _e(t);
    }
    function Rn() {
        if (typeof Reflect == "undefined" || !Reflect.construct || Reflect.construct.sham) return !1;
        if (typeof Proxy == "function") return !0;
        try {
            return Date.prototype.toString.call(Reflect.construct(Date, [], function() {})), !0;
        } catch (t) {
            return !1;
        }
    }
    function le(t, e, r) {
        return Rn() ? le = Reflect.construct : le = function(o, i, a) {
            var c = [
                null
            ];
            c.push.apply(c, i);
            var u = Function.bind.apply(o, c), f = new u;
            return a && ee(f, a.prototype), f;
        }, le.apply(null, arguments);
    }
    function In(t) {
        return Function.toString.call(t).indexOf("[native code]") !== -1;
    }
    function ee(t, e) {
        return ee = Object.setPrototypeOf || function(n, o) {
            return n.__proto__ = o, n;
        }, ee(t, e);
    }
    function R(t) {
        return R = Object.setPrototypeOf ? Object.getPrototypeOf : function(r) {
            return r.__proto__ || Object.getPrototypeOf(r);
        }, R(t);
    }
    function S(t) {
        return typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? S = function(r) {
            return typeof r;
        } : S = function(r) {
            return r && typeof Symbol == "function" && r.constructor === Symbol && r !== Symbol.prototype ? "symbol" : typeof r;
        }, S(t);
    }
    var Nn = ii, De = Nn.inspect, xn = Fe(), Fn = xn.codes.ERR_INVALID_ARG_TYPE;
    function ht(t, e, r) {
        return (r === void 0 || r > t.length) && (r = t.length), t.substring(r - e.length, r) === e;
    }
    function Tn(t, e) {
        if (e = Math.floor(e), t.length == 0 || e == 0) return "";
        var r = t.length * e;
        for(e = Math.floor(Math.log(e) / Math.log(2)); e;)t += t, e--;
        return t += t.substring(0, r - t.length), t;
    }
    var P = "", Q = "", Z = "", b = "", _ = {
        deepStrictEqual: "Expected values to be strictly deep-equal:",
        strictEqual: "Expected values to be strictly equal:",
        strictEqualObject: 'Expected "actual" to be reference-equal to "expected":',
        deepEqual: "Expected values to be loosely deep-equal:",
        equal: "Expected values to be loosely equal:",
        notDeepStrictEqual: 'Expected "actual" not to be strictly deep-equal to:',
        notStrictEqual: 'Expected "actual" to be strictly unequal to:',
        notStrictEqualObject: 'Expected "actual" not to be reference-equal to "expected":',
        notDeepEqual: 'Expected "actual" not to be loosely deep-equal to:',
        notEqual: 'Expected "actual" to be loosely unequal to:',
        notIdentical: "Values identical but not reference-equal:"
    }, _n = 10;
    function dt(t) {
        var e = Object.keys(t), r = Object.create(Object.getPrototypeOf(t));
        return e.forEach(function(n) {
            r[n] = t[n];
        }), Object.defineProperty(r, "message", {
            value: t.message
        }), r;
    }
    function K(t) {
        return De(t, {
            compact: !1,
            customInspect: !1,
            depth: 1e3,
            maxArrayLength: 1 / 0,
            showHidden: !1,
            breakLength: 1 / 0,
            showProxy: !1,
            sorted: !0,
            getters: !0
        });
    }
    function Dn(t, e, r) {
        var n = "", o = "", i = 0, a = "", c = !1, u = K(t), f = u.split(`
`), l = K(e).split(`
`), s = 0, p = "";
        if (r === "strictEqual" && S(t) === "object" && S(e) === "object" && t !== null && e !== null && (r = "strictEqualObject"), f.length === 1 && l.length === 1 && f[0] !== l[0]) {
            var d = f[0].length + l[0].length;
            if (d <= _n) {
                if ((S(t) !== "object" || t === null) && (S(e) !== "object" || e === null) && (t !== 0 || e !== 0)) return "".concat(_[r], `

`) + "".concat(f[0], " !== ").concat(l[0], `
`);
            } else if (r !== "strictEqualObject") {
                var v = __default.stderr && __default.stderr.isTTY ? __default.stderr.columns : 80;
                if (d < v) {
                    for(; f[0][s] === l[0][s];)s++;
                    s > 2 && (p = `
  `.concat(Tn(" ", s), "^"), s = 0);
                }
            }
        }
        for(var w = f[f.length - 1], ot = l[l.length - 1]; w === ot && (s++ < 2 ? a = `
  `.concat(w).concat(a) : n = w, f.pop(), l.pop(), !(f.length === 0 || l.length === 0));)w = f[f.length - 1], ot = l[l.length - 1];
        var qe = Math.max(f.length, l.length);
        if (qe === 0) {
            var C = u.split(`
`);
            if (C.length > 30) for(C[26] = "".concat(P, "...").concat(b); C.length > 27;)C.pop();
            return "".concat(_.notIdentical, `

`).concat(C.join(`
`), `
`);
        }
        s > 3 && (a = `
`.concat(P, "...").concat(b).concat(a), c = !0), n !== "" && (a = `
  `.concat(n).concat(a), n = "");
        var A = 0, it = _[r] + `
`.concat(Q, "+ actual").concat(b, " ").concat(Z, "- expected").concat(b), at = " ".concat(P, "...").concat(b, " Lines skipped");
        for(s = 0; s < qe; s++){
            var q = s - i;
            if (f.length < s + 1) q > 1 && s > 2 && (q > 4 ? (o += `
`.concat(P, "...").concat(b), c = !0) : q > 3 && (o += `
  `.concat(l[s - 2]), A++), o += `
  `.concat(l[s - 1]), A++), i = s, n += `
`.concat(Z, "-").concat(b, " ").concat(l[s]), A++;
            else if (l.length < s + 1) q > 1 && s > 2 && (q > 4 ? (o += `
`.concat(P, "...").concat(b), c = !0) : q > 3 && (o += `
  `.concat(f[s - 2]), A++), o += `
  `.concat(f[s - 1]), A++), i = s, o += `
`.concat(Q, "+").concat(b, " ").concat(f[s]), A++;
            else {
                var H = l[s], T = f[s], Re = T !== H && (!ht(T, ",") || T.slice(0, -1) !== H);
                Re && ht(H, ",") && H.slice(0, -1) === T && (Re = !1, T += ","), Re ? (q > 1 && s > 2 && (q > 4 ? (o += `
`.concat(P, "...").concat(b), c = !0) : q > 3 && (o += `
  `.concat(f[s - 2]), A++), o += `
  `.concat(f[s - 1]), A++), i = s, o += `
`.concat(Q, "+").concat(b, " ").concat(T), n += `
`.concat(Z, "-").concat(b, " ").concat(H), A += 2) : (o += n, n = "", (q === 1 || s === 0) && (o += `
  `.concat(T), A++));
            }
            if (A > 20 && s < qe - 2) return "".concat(it).concat(at, `
`).concat(o, `
`).concat(P, "...").concat(b).concat(n, `
`) + "".concat(P, "...").concat(b);
        }
        return "".concat(it).concat(c ? at : "", `
`).concat(o).concat(n).concat(a).concat(p);
    }
    var Un = function(t) {
        qn(e, t);
        function e(r) {
            var n;
            if (Pn(this, e), S(r) !== "object" || r === null) throw new Fn("options", "Object", r);
            var o = r.message, i = r.operator, a = r.stackStartFn, c = r.actual, u = r.expected, f = Error.stackTraceLimit;
            if (Error.stackTraceLimit = 0, o != null) n = B(this, R(e).call(this, String(o)));
            else if (__default.stderr && __default.stderr.isTTY && (__default.stderr && __default.stderr.getColorDepth && __default.stderr.getColorDepth() !== 1 ? (P = "\x1B[34m", Q = "\x1B[32m", b = "\x1B[39m", Z = "\x1B[31m") : (P = "", Q = "", b = "", Z = "")), S(c) === "object" && c !== null && S(u) === "object" && u !== null && "stack" in c && c instanceof Error && "stack" in u && u instanceof Error && (c = dt(c), u = dt(u)), i === "deepStrictEqual" || i === "strictEqual") n = B(this, R(e).call(this, Dn(c, u, i)));
            else if (i === "notDeepStrictEqual" || i === "notStrictEqual") {
                var l = _[i], s = K(c).split(`
`);
                if (i === "notStrictEqual" && S(c) === "object" && c !== null && (l = _.notStrictEqualObject), s.length > 30) for(s[26] = "".concat(P, "...").concat(b); s.length > 27;)s.pop();
                s.length === 1 ? n = B(this, R(e).call(this, "".concat(l, " ").concat(s[0]))) : n = B(this, R(e).call(this, "".concat(l, `

`).concat(s.join(`
`), `
`)));
            } else {
                var p = K(c), d = "", v = _[i];
                i === "notDeepEqual" || i === "notEqual" ? (p = "".concat(_[i], `

`).concat(p), p.length > 1024 && (p = "".concat(p.slice(0, 1021), "..."))) : (d = "".concat(K(u)), p.length > 512 && (p = "".concat(p.slice(0, 509), "...")), d.length > 512 && (d = "".concat(d.slice(0, 509), "...")), i === "deepEqual" || i === "equal" ? p = "".concat(v, `

`).concat(p, `

should equal

`) : d = " ".concat(i, " ").concat(d)), n = B(this, R(e).call(this, "".concat(p).concat(d)));
            }
            return Error.stackTraceLimit = f, n.generatedMessage = !o, Object.defineProperty(Te(n), "name", {
                value: "AssertionError [ERR_ASSERTION]",
                enumerable: !1,
                writable: !0,
                configurable: !0
            }), n.code = "ERR_ASSERTION", n.actual = c, n.expected = u, n.operator = i, Error.captureStackTrace && Error.captureStackTrace(Te(n), a), n.stack, n.name = "AssertionError", B(n);
        }
        return jn(e, [
            {
                key: "toString",
                value: function() {
                    return "".concat(this.name, " [").concat(this.code, "]: ").concat(this.message);
                }
            },
            {
                key: De.custom,
                value: function(n, o) {
                    return De(this, wn({}, o, {
                        customInspect: !1,
                        depth: 0
                    }));
                }
            }
        ]), e;
    }(_e(Error));
    bt.exports = Un;
});
var St = g((li, Et)=>{
    "use strict";
    function mt(t, e) {
        if (t == null) throw new TypeError("Cannot convert first argument to object");
        for(var r = Object(t), n = 1; n < arguments.length; n++){
            var o = arguments[n];
            if (o != null) for(var i = Object.keys(Object(o)), a = 0, c = i.length; a < c; a++){
                var u = i[a], f = Object.getOwnPropertyDescriptor(o, u);
                f !== void 0 && f.enumerable && (r[u] = o[u]);
            }
        }
        return r;
    }
    function Mn() {
        Object.assign || Object.defineProperty(Object, "assign", {
            enumerable: !1,
            configurable: !0,
            writable: !0,
            value: mt
        });
    }
    Et.exports = {
        assign: mt,
        polyfill: Mn
    };
});
var Ue = g((pi, wt)=>{
    "use strict";
    var Ot = Object.prototype.toString;
    wt.exports = function(e) {
        var r = Ot.call(e), n = r === "[object Arguments]";
        return n || (n = r !== "[object Array]" && e !== null && typeof e == "object" && typeof e.length == "number" && e.length >= 0 && Ot.call(e.callee) === "[object Function]"), n;
    };
});
var Ft = g((yi, xt)=>{
    "use strict";
    var Nt;
    Object.keys || (te = Object.prototype.hasOwnProperty, Me = Object.prototype.toString, At = Ue(), $e = Object.prototype.propertyIsEnumerable, Pt = !$e.call({
        toString: null
    }, "toString"), jt = $e.call(function() {}, "prototype"), re = [
        "toString",
        "toLocaleString",
        "valueOf",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "constructor"
    ], pe = function(t) {
        var e = t.constructor;
        return e && e.prototype === t;
    }, qt = {
        $applicationCache: !0,
        $console: !0,
        $external: !0,
        $frame: !0,
        $frameElement: !0,
        $frames: !0,
        $innerHeight: !0,
        $innerWidth: !0,
        $onmozfullscreenchange: !0,
        $onmozfullscreenerror: !0,
        $outerHeight: !0,
        $outerWidth: !0,
        $pageXOffset: !0,
        $pageYOffset: !0,
        $parent: !0,
        $scrollLeft: !0,
        $scrollTop: !0,
        $scrollX: !0,
        $scrollY: !0,
        $self: !0,
        $webkitIndexedDB: !0,
        $webkitStorageInfo: !0,
        $window: !0
    }, Rt = function() {
        if (typeof window == "undefined") return !1;
        for(var t in window)try {
            if (!qt["$" + t] && te.call(window, t) && window[t] !== null && typeof window[t] == "object") try {
                pe(window[t]);
            } catch (e) {
                return !0;
            }
        } catch (e1) {
            return !0;
        }
        return !1;
    }(), It = function(t) {
        if (typeof window == "undefined" || !Rt) return pe(t);
        try {
            return pe(t);
        } catch (e) {
            return !1;
        }
    }, Nt = function(e) {
        var r = e !== null && typeof e == "object", n = Me.call(e) === "[object Function]", o = At(e), i = r && Me.call(e) === "[object String]", a = [];
        if (!r && !n && !o) throw new TypeError("Object.keys called on a non-object");
        var c = jt && n;
        if (i && e.length > 0 && !te.call(e, 0)) for(var u = 0; u < e.length; ++u)a.push(String(u));
        if (o && e.length > 0) for(var f = 0; f < e.length; ++f)a.push(String(f));
        else for(var l in e)!(c && l === "prototype") && te.call(e, l) && a.push(String(l));
        if (Pt) for(var s = It(e), p = 0; p < re.length; ++p)!(s && re[p] === "constructor") && te.call(e, re[p]) && a.push(re[p]);
        return a;
    });
    var te, Me, At, $e, Pt, jt, re, pe, qt, Rt, It;
    xt.exports = Nt;
});
var Ut = g((gi, Dt)=>{
    "use strict";
    var $n = Array.prototype.slice, Gn = Ue(), Tt = Object.keys, ye = Tt ? function(e) {
        return Tt(e);
    } : Ft(), _t = Object.keys;
    ye.shim = function() {
        if (Object.keys) {
            var e = function() {
                var r = Object.keys(arguments);
                return r && r.length === arguments.length;
            }(1, 2);
            e || (Object.keys = function(n) {
                return Gn(n) ? _t($n.call(n)) : _t(n);
            });
        } else Object.keys = ye;
        return Object.keys || ye;
    };
    Dt.exports = ye;
});
var $t = g((hi, Mt)=>{
    "use strict";
    Mt.exports = function() {
        if (typeof Symbol != "function" || typeof Object.getOwnPropertySymbols != "function") return !1;
        if (typeof Symbol.iterator == "symbol") return !0;
        var e = {}, r = Symbol("test"), n = Object(r);
        if (typeof r == "string" || Object.prototype.toString.call(r) !== "[object Symbol]" || Object.prototype.toString.call(n) !== "[object Symbol]") return !1;
        var o = 42;
        e[r] = o;
        for(r in e)return !1;
        if (typeof Object.keys == "function" && Object.keys(e).length !== 0 || typeof Object.getOwnPropertyNames == "function" && Object.getOwnPropertyNames(e).length !== 0) return !1;
        var i = Object.getOwnPropertySymbols(e);
        if (i.length !== 1 || i[0] !== r || !Object.prototype.propertyIsEnumerable.call(e, r)) return !1;
        if (typeof Object.getOwnPropertyDescriptor == "function") {
            var a = Object.getOwnPropertyDescriptor(e, r);
            if (a.value !== o || a.enumerable !== !0) return !1;
        }
        return !0;
    };
});
var Lt = g((di, Bt)=>{
    "use strict";
    var Gt = typeof Symbol != "undefined" && Symbol, Bn = $t();
    Bt.exports = function() {
        return typeof Gt != "function" || typeof Symbol != "function" || typeof Gt("foo") != "symbol" || typeof Symbol("bar") != "symbol" ? !1 : Bn();
    };
});
var kt = g((bi, Vt)=>{
    "use strict";
    var Ln = "Function.prototype.bind called on incompatible ", Ge = Array.prototype.slice, Vn = Object.prototype.toString, kn = "[object Function]";
    Vt.exports = function(e) {
        var r = this;
        if (typeof r != "function" || Vn.call(r) !== kn) throw new TypeError(Ln + r);
        for(var n = Ge.call(arguments, 1), o, i = function() {
            if (this instanceof o) {
                var l = r.apply(this, n.concat(Ge.call(arguments)));
                return Object(l) === l ? l : this;
            } else return r.apply(e, n.concat(Ge.call(arguments)));
        }, a = Math.max(0, r.length - n.length), c = [], u = 0; u < a; u++)c.push("$" + u);
        if (o = Function("binder", "return function (" + c.join(",") + "){ return binder.apply(this,arguments); }")(i), r.prototype) {
            var f = function() {};
            f.prototype = r.prototype, o.prototype = new f, f.prototype = null;
        }
        return o;
    };
});
var ge = g((vi, Wt)=>{
    "use strict";
    var Wn = kt();
    Wt.exports = Function.prototype.bind || Wn;
});
var Yt = g((mi, zt)=>{
    "use strict";
    var zn = ge();
    zt.exports = zn.call(Function.call, Object.prototype.hasOwnProperty);
});
var Ve = g((Ei, Xt)=>{
    "use strict";
    var y, z = SyntaxError, Jt = Function, k = TypeError, Be = function(t) {
        try {
            return Jt('"use strict"; return (' + t + ").constructor;")();
        } catch (e) {}
    }, D = Object.getOwnPropertyDescriptor;
    if (D) try {
        D({}, "");
    } catch (t) {
        D = null;
    }
    var Le = function() {
        throw new k;
    }, Yn = D ? function() {
        try {
            return arguments.callee, Le;
        } catch (t) {
            try {
                return D(arguments, "callee").get;
            } catch (e) {
                return Le;
            }
        }
    }() : Le, L = Lt()(), I = Object.getPrototypeOf || function(t) {
        return t.__proto__;
    }, V = {}, Cn = typeof Uint8Array == "undefined" ? y : I(Uint8Array), W = {
        "%AggregateError%": typeof AggregateError == "undefined" ? y : AggregateError,
        "%Array%": Array,
        "%ArrayBuffer%": typeof ArrayBuffer == "undefined" ? y : ArrayBuffer,
        "%ArrayIteratorPrototype%": L ? I([][Symbol.iterator]()) : y,
        "%AsyncFromSyncIteratorPrototype%": y,
        "%AsyncFunction%": V,
        "%AsyncGenerator%": V,
        "%AsyncGeneratorFunction%": V,
        "%AsyncIteratorPrototype%": V,
        "%Atomics%": typeof Atomics == "undefined" ? y : Atomics,
        "%BigInt%": typeof BigInt == "undefined" ? y : BigInt,
        "%Boolean%": Boolean,
        "%DataView%": typeof DataView == "undefined" ? y : DataView,
        "%Date%": Date,
        "%decodeURI%": decodeURI,
        "%decodeURIComponent%": decodeURIComponent,
        "%encodeURI%": encodeURI,
        "%encodeURIComponent%": encodeURIComponent,
        "%Error%": Error,
        "%eval%": eval,
        "%EvalError%": EvalError,
        "%Float32Array%": typeof Float32Array == "undefined" ? y : Float32Array,
        "%Float64Array%": typeof Float64Array == "undefined" ? y : Float64Array,
        "%FinalizationRegistry%": typeof FinalizationRegistry == "undefined" ? y : FinalizationRegistry,
        "%Function%": Jt,
        "%GeneratorFunction%": V,
        "%Int8Array%": typeof Int8Array == "undefined" ? y : Int8Array,
        "%Int16Array%": typeof Int16Array == "undefined" ? y : Int16Array,
        "%Int32Array%": typeof Int32Array == "undefined" ? y : Int32Array,
        "%isFinite%": isFinite,
        "%isNaN%": isNaN,
        "%IteratorPrototype%": L ? I(I([][Symbol.iterator]())) : y,
        "%JSON%": typeof JSON == "object" ? JSON : y,
        "%Map%": typeof Map == "undefined" ? y : Map,
        "%MapIteratorPrototype%": typeof Map == "undefined" || !L ? y : I(new Map()[Symbol.iterator]()),
        "%Math%": Math,
        "%Number%": Number,
        "%Object%": Object,
        "%parseFloat%": parseFloat,
        "%parseInt%": parseInt,
        "%Promise%": typeof Promise == "undefined" ? y : Promise,
        "%Proxy%": typeof Proxy == "undefined" ? y : Proxy,
        "%RangeError%": RangeError,
        "%ReferenceError%": ReferenceError,
        "%Reflect%": typeof Reflect == "undefined" ? y : Reflect,
        "%RegExp%": RegExp,
        "%Set%": typeof Set == "undefined" ? y : Set,
        "%SetIteratorPrototype%": typeof Set == "undefined" || !L ? y : I(new Set()[Symbol.iterator]()),
        "%SharedArrayBuffer%": typeof SharedArrayBuffer == "undefined" ? y : SharedArrayBuffer,
        "%String%": String,
        "%StringIteratorPrototype%": L ? I(""[Symbol.iterator]()) : y,
        "%Symbol%": L ? Symbol : y,
        "%SyntaxError%": z,
        "%ThrowTypeError%": Yn,
        "%TypedArray%": Cn,
        "%TypeError%": k,
        "%Uint8Array%": typeof Uint8Array == "undefined" ? y : Uint8Array,
        "%Uint8ClampedArray%": typeof Uint8ClampedArray == "undefined" ? y : Uint8ClampedArray,
        "%Uint16Array%": typeof Uint16Array == "undefined" ? y : Uint16Array,
        "%Uint32Array%": typeof Uint32Array == "undefined" ? y : Uint32Array,
        "%URIError%": URIError,
        "%WeakMap%": typeof WeakMap == "undefined" ? y : WeakMap,
        "%WeakRef%": typeof WeakRef == "undefined" ? y : WeakRef,
        "%WeakSet%": typeof WeakSet == "undefined" ? y : WeakSet
    }, Hn = function t(e) {
        var r;
        if (e === "%AsyncFunction%") r = Be("async function () {}");
        else if (e === "%GeneratorFunction%") r = Be("function* () {}");
        else if (e === "%AsyncGeneratorFunction%") r = Be("async function* () {}");
        else if (e === "%AsyncGenerator%") {
            var n = t("%AsyncGeneratorFunction%");
            n && (r = n.prototype);
        } else if (e === "%AsyncIteratorPrototype%") {
            var o = t("%AsyncGenerator%");
            o && (r = I(o.prototype));
        }
        return W[e] = r, r;
    }, Ct = {
        "%ArrayBufferPrototype%": [
            "ArrayBuffer",
            "prototype"
        ],
        "%ArrayPrototype%": [
            "Array",
            "prototype"
        ],
        "%ArrayProto_entries%": [
            "Array",
            "prototype",
            "entries"
        ],
        "%ArrayProto_forEach%": [
            "Array",
            "prototype",
            "forEach"
        ],
        "%ArrayProto_keys%": [
            "Array",
            "prototype",
            "keys"
        ],
        "%ArrayProto_values%": [
            "Array",
            "prototype",
            "values"
        ],
        "%AsyncFunctionPrototype%": [
            "AsyncFunction",
            "prototype"
        ],
        "%AsyncGenerator%": [
            "AsyncGeneratorFunction",
            "prototype"
        ],
        "%AsyncGeneratorPrototype%": [
            "AsyncGeneratorFunction",
            "prototype",
            "prototype"
        ],
        "%BooleanPrototype%": [
            "Boolean",
            "prototype"
        ],
        "%DataViewPrototype%": [
            "DataView",
            "prototype"
        ],
        "%DatePrototype%": [
            "Date",
            "prototype"
        ],
        "%ErrorPrototype%": [
            "Error",
            "prototype"
        ],
        "%EvalErrorPrototype%": [
            "EvalError",
            "prototype"
        ],
        "%Float32ArrayPrototype%": [
            "Float32Array",
            "prototype"
        ],
        "%Float64ArrayPrototype%": [
            "Float64Array",
            "prototype"
        ],
        "%FunctionPrototype%": [
            "Function",
            "prototype"
        ],
        "%Generator%": [
            "GeneratorFunction",
            "prototype"
        ],
        "%GeneratorPrototype%": [
            "GeneratorFunction",
            "prototype",
            "prototype"
        ],
        "%Int8ArrayPrototype%": [
            "Int8Array",
            "prototype"
        ],
        "%Int16ArrayPrototype%": [
            "Int16Array",
            "prototype"
        ],
        "%Int32ArrayPrototype%": [
            "Int32Array",
            "prototype"
        ],
        "%JSONParse%": [
            "JSON",
            "parse"
        ],
        "%JSONStringify%": [
            "JSON",
            "stringify"
        ],
        "%MapPrototype%": [
            "Map",
            "prototype"
        ],
        "%NumberPrototype%": [
            "Number",
            "prototype"
        ],
        "%ObjectPrototype%": [
            "Object",
            "prototype"
        ],
        "%ObjProto_toString%": [
            "Object",
            "prototype",
            "toString"
        ],
        "%ObjProto_valueOf%": [
            "Object",
            "prototype",
            "valueOf"
        ],
        "%PromisePrototype%": [
            "Promise",
            "prototype"
        ],
        "%PromiseProto_then%": [
            "Promise",
            "prototype",
            "then"
        ],
        "%Promise_all%": [
            "Promise",
            "all"
        ],
        "%Promise_reject%": [
            "Promise",
            "reject"
        ],
        "%Promise_resolve%": [
            "Promise",
            "resolve"
        ],
        "%RangeErrorPrototype%": [
            "RangeError",
            "prototype"
        ],
        "%ReferenceErrorPrototype%": [
            "ReferenceError",
            "prototype"
        ],
        "%RegExpPrototype%": [
            "RegExp",
            "prototype"
        ],
        "%SetPrototype%": [
            "Set",
            "prototype"
        ],
        "%SharedArrayBufferPrototype%": [
            "SharedArrayBuffer",
            "prototype"
        ],
        "%StringPrototype%": [
            "String",
            "prototype"
        ],
        "%SymbolPrototype%": [
            "Symbol",
            "prototype"
        ],
        "%SyntaxErrorPrototype%": [
            "SyntaxError",
            "prototype"
        ],
        "%TypedArrayPrototype%": [
            "TypedArray",
            "prototype"
        ],
        "%TypeErrorPrototype%": [
            "TypeError",
            "prototype"
        ],
        "%Uint8ArrayPrototype%": [
            "Uint8Array",
            "prototype"
        ],
        "%Uint8ClampedArrayPrototype%": [
            "Uint8ClampedArray",
            "prototype"
        ],
        "%Uint16ArrayPrototype%": [
            "Uint16Array",
            "prototype"
        ],
        "%Uint32ArrayPrototype%": [
            "Uint32Array",
            "prototype"
        ],
        "%URIErrorPrototype%": [
            "URIError",
            "prototype"
        ],
        "%WeakMapPrototype%": [
            "WeakMap",
            "prototype"
        ],
        "%WeakSetPrototype%": [
            "WeakSet",
            "prototype"
        ]
    }, ne = ge(), he = Yt(), Jn = ne.call(Function.call, Array.prototype.concat), Xn = ne.call(Function.apply, Array.prototype.splice), Ht = ne.call(Function.call, String.prototype.replace), de = ne.call(Function.call, String.prototype.slice), Qn = ne.call(Function.call, RegExp.prototype.exec), Zn = /[^%.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|%$))/g, Kn = /\\(\\)?/g, eo = function(e) {
        var r = de(e, 0, 1), n = de(e, -1);
        if (r === "%" && n !== "%") throw new z("invalid intrinsic syntax, expected closing `%`");
        if (n === "%" && r !== "%") throw new z("invalid intrinsic syntax, expected opening `%`");
        var o = [];
        return Ht(e, Zn, function(i, a, c, u) {
            o[o.length] = c ? Ht(u, Kn, "$1") : a || i;
        }), o;
    }, to = function(e, r) {
        var n = e, o;
        if (he(Ct, n) && (o = Ct[n], n = "%" + o[0] + "%"), he(W, n)) {
            var i = W[n];
            if (i === V && (i = Hn(n)), typeof i == "undefined" && !r) throw new k("intrinsic " + e + " exists, but is not available. Please file an issue!");
            return {
                alias: o,
                name: n,
                value: i
            };
        }
        throw new z("intrinsic " + e + " does not exist!");
    };
    Xt.exports = function(e, r) {
        if (typeof e != "string" || e.length === 0) throw new k("intrinsic name must be a non-empty string");
        if (arguments.length > 1 && typeof r != "boolean") throw new k('"allowMissing" argument must be a boolean');
        if (Qn(/^%?[^%]*%?$/, e) === null) throw new z("`%` may not be present anywhere but at the beginning and end of the intrinsic name");
        var n = eo(e), o = n.length > 0 ? n[0] : "", i = to("%" + o + "%", r), a = i.name, c = i.value, u = !1, f = i.alias;
        f && (o = f[0], Xn(n, Jn([
            0,
            1
        ], f)));
        for(var l = 1, s = !0; l < n.length; l += 1){
            var p = n[l], d = de(p, 0, 1), v = de(p, -1);
            if ((d === '"' || d === "'" || d === "`" || v === '"' || v === "'" || v === "`") && d !== v) throw new z("property names with quotes must have matching quotes");
            if ((p === "constructor" || !s) && (u = !0), o += "." + p, a = "%" + o + "%", he(W, a)) c = W[a];
            else if (c != null) {
                if (!(p in c)) {
                    if (!r) throw new k("base intrinsic for " + e + " exists, but the property is not available.");
                    return;
                }
                if (D && l + 1 >= n.length) {
                    var w = D(c, p);
                    s = !!w, s && "get" in w && !("originalValue" in w.get) ? c = w.get : c = c[p];
                } else s = he(c, p), c = c[p];
                s && !u && (W[a] = c);
            }
        }
        return c;
    };
});
var Zt = g((Si, Qt)=>{
    "use strict";
    var ro = Ve(), ke = ro("%Object.defineProperty%", !0), We = function() {
        if (ke) try {
            return ke({}, "a", {
                value: 1
            }), !0;
        } catch (e) {
            return !1;
        }
        return !1;
    };
    We.hasArrayLengthDefineBug = function() {
        if (!We()) return null;
        try {
            return ke([], "length", {
                value: 1
            }).length !== 1;
        } catch (e) {
            return !0;
        }
    };
    Qt.exports = We;
});
var oe = g((Oi, rr)=>{
    "use strict";
    var no = Ut(), oo = typeof Symbol == "function" && typeof Symbol("foo") == "symbol", io = Object.prototype.toString, ao = Array.prototype.concat, Kt = Object.defineProperty, co = function(t) {
        return typeof t == "function" && io.call(t) === "[object Function]";
    }, uo = Zt()(), er = Kt && uo, fo = function(t, e, r, n) {
        e in t && (!co(n) || !n()) || (er ? Kt(t, e, {
            configurable: !0,
            enumerable: !1,
            value: r,
            writable: !0
        }) : t[e] = r);
    }, tr = function(t, e) {
        var r = arguments.length > 2 ? arguments[2] : {}, n = no(e);
        oo && (n = ao.call(n, Object.getOwnPropertySymbols(e)));
        for(var o = 0; o < n.length; o += 1)fo(t, n[o], e[n[o]], r[n[o]]);
    };
    tr.supportsDescriptors = !!er;
    rr.exports = tr;
});
var Ye = g((wi, be)=>{
    "use strict";
    var ze = ge(), Y = Ve(), ir = Y("%Function.prototype.apply%"), ar = Y("%Function.prototype.call%"), cr = Y("%Reflect.apply%", !0) || ze.call(ar, ir), nr = Y("%Object.getOwnPropertyDescriptor%", !0), U = Y("%Object.defineProperty%", !0), so = Y("%Math.max%");
    if (U) try {
        U({}, "a", {
            value: 1
        });
    } catch (t) {
        U = null;
    }
    be.exports = function(e) {
        var r = cr(ze, ar, arguments);
        if (nr && U) {
            var n = nr(r, "length");
            n.configurable && U(r, "length", {
                value: 1 + so(0, e.length - (arguments.length - 1))
            });
        }
        return r;
    };
    var or = function() {
        return cr(ze, ir, arguments);
    };
    U ? U(be.exports, "apply", {
        value: or
    }) : be.exports.apply = or;
});
var Ce = g((Ai, fr)=>{
    "use strict";
    var ur = function(t) {
        return t !== t;
    };
    fr.exports = function(e, r) {
        return e === 0 && r === 0 ? 1 / e === 1 / r : !!(e === r || ur(e) && ur(r));
    };
});
var He = g((Pi, sr)=>{
    "use strict";
    var lo = Ce();
    sr.exports = function() {
        return typeof Object.is == "function" ? Object.is : lo;
    };
});
var pr1 = g((ji, lr)=>{
    "use strict";
    var po = He(), yo = oe();
    lr.exports = function() {
        var e = po();
        return yo(Object, {
            is: e
        }, {
            is: function() {
                return Object.is !== e;
            }
        }), e;
    };
});
var Je = g((qi, hr)=>{
    "use strict";
    var go = oe(), ho = Ye(), bo = Ce(), yr = He(), vo = pr1(), gr = ho(yr(), Object);
    go(gr, {
        getPolyfill: yr,
        implementation: bo,
        shim: vo
    });
    hr.exports = gr;
});
var Xe1 = g((Ri, dr)=>{
    "use strict";
    dr.exports = function(e) {
        return e !== e;
    };
});
var Qe = g((Ii, br)=>{
    "use strict";
    var mo = Xe1();
    br.exports = function() {
        return Number.isNaN && Number.isNaN(NaN) && !Number.isNaN("a") ? Number.isNaN : mo;
    };
});
var mr = g((Ni, vr)=>{
    "use strict";
    var Eo = oe(), So = Qe();
    vr.exports = function() {
        var e = So();
        return Eo(Number, {
            isNaN: e
        }, {
            isNaN: function() {
                return Number.isNaN !== e;
            }
        }), e;
    };
});
var wr = g((xi, Or)=>{
    "use strict";
    var Oo = Ye(), wo = oe(), Ao = Xe1(), Er = Qe(), Po = mr(), Sr = Oo(Er(), Number);
    wo(Sr, {
        getPolyfill: Er,
        implementation: Ao,
        shim: Po
    });
    Or.exports = Sr;
});
var Vr = g((Fi, Lr)=>{
    "use strict";
    function Ar(t, e) {
        return Ro(t) || qo(t, e) || jo();
    }
    function jo() {
        throw new TypeError("Invalid attempt to destructure non-iterable instance");
    }
    function qo(t, e) {
        var r = [], n = !0, o = !1, i = void 0;
        try {
            for(var a = t[Symbol.iterator](), c; !(n = (c = a.next()).done) && (r.push(c.value), !(e && r.length === e)); n = !0);
        } catch (u) {
            o = !0, i = u;
        } finally{
            try {
                !n && a.return != null && a.return();
            } finally{
                if (o) throw i;
            }
        }
        return r;
    }
    function Ro(t) {
        if (Array.isArray(t)) return t;
    }
    function E(t) {
        return typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? E = function(r) {
            return typeof r;
        } : E = function(r) {
            return r && typeof Symbol == "function" && r.constructor === Symbol && r !== Symbol.prototype ? "symbol" : typeof r;
        }, E(t);
    }
    var Io = /a/g.flags !== void 0, Ae = function(e) {
        var r = [];
        return e.forEach(function(n) {
            return r.push(n);
        }), r;
    }, Pr = function(e) {
        var r = [];
        return e.forEach(function(n, o) {
            return r.push([
                o,
                n
            ]);
        }), r;
    }, Ur = Object.is ? Object.is : Je(), Oe = Object.getOwnPropertySymbols ? Object.getOwnPropertySymbols : function() {
        return [];
    }, Ze = Number.isNaN ? Number.isNaN : wr();
    function et(t) {
        return t.call.bind(t);
    }
    var ae = et(Object.prototype.hasOwnProperty), we = et(Object.prototype.propertyIsEnumerable), jr = et(Object.prototype.toString), m = ii.types, No = m.isAnyArrayBuffer, xo = m.isArrayBufferView, qr = m.isDate, ve = m.isMap, Rr = m.isRegExp, me = m.isSet, Fo = m.isNativeError, To = m.isBoxedPrimitive, Ir = m.isNumberObject, Nr = m.isStringObject, xr = m.isBooleanObject, Fr = m.isBigIntObject, _o = m.isSymbolObject, Do = m.isFloat32Array, Uo = m.isFloat64Array;
    function Mo(t) {
        if (t.length === 0 || t.length > 10) return !0;
        for(var e = 0; e < t.length; e++){
            var r = t.charCodeAt(e);
            if (r < 48 || r > 57) return !0;
        }
        return t.length === 10 && t >= Math.pow(2, 32);
    }
    function Ee(t) {
        return Object.keys(t).filter(Mo).concat(Oe(t).filter(Object.prototype.propertyIsEnumerable.bind(t)));
    }
    function Mr(t, e) {
        if (t === e) return 0;
        for(var r = t.length, n = e.length, o = 0, i = Math.min(r, n); o < i; ++o)if (t[o] !== e[o]) {
            r = t[o], n = e[o];
            break;
        }
        return r < n ? -1 : n < r ? 1 : 0;
    }
    var Se = void 0, $o = !0, Go = !1, Ke = 0, tt = 1, $r = 2, Gr = 3;
    function Bo(t, e) {
        return Io ? t.source === e.source && t.flags === e.flags : RegExp.prototype.toString.call(t) === RegExp.prototype.toString.call(e);
    }
    function Lo(t, e) {
        if (t.byteLength !== e.byteLength) return !1;
        for(var r = 0; r < t.byteLength; r++)if (t[r] !== e[r]) return !1;
        return !0;
    }
    function Vo(t, e) {
        return t.byteLength !== e.byteLength ? !1 : Mr(new Uint8Array(t.buffer, t.byteOffset, t.byteLength), new Uint8Array(e.buffer, e.byteOffset, e.byteLength)) === 0;
    }
    function ko(t, e) {
        return t.byteLength === e.byteLength && Mr(new Uint8Array(t), new Uint8Array(e)) === 0;
    }
    function Wo(t, e) {
        return Ir(t) ? Ir(e) && Ur(Number.prototype.valueOf.call(t), Number.prototype.valueOf.call(e)) : Nr(t) ? Nr(e) && String.prototype.valueOf.call(t) === String.prototype.valueOf.call(e) : xr(t) ? xr(e) && Boolean.prototype.valueOf.call(t) === Boolean.prototype.valueOf.call(e) : Fr(t) ? Fr(e) && BigInt.prototype.valueOf.call(t) === BigInt.prototype.valueOf.call(e) : _o(e) && Symbol.prototype.valueOf.call(t) === Symbol.prototype.valueOf.call(e);
    }
    function O(t, e, r, n) {
        if (t === e) return t !== 0 ? !0 : r ? Ur(t, e) : !0;
        if (r) {
            if (E(t) !== "object") return typeof t == "number" && Ze(t) && Ze(e);
            if (E(e) !== "object" || t === null || e === null || Object.getPrototypeOf(t) !== Object.getPrototypeOf(e)) return !1;
        } else {
            if (t === null || E(t) !== "object") return e === null || E(e) !== "object" ? t == e : !1;
            if (e === null || E(e) !== "object") return !1;
        }
        var o = jr(t), i = jr(e);
        if (o !== i) return !1;
        if (Array.isArray(t)) {
            if (t.length !== e.length) return !1;
            var a = Ee(t, Se), c = Ee(e, Se);
            return a.length !== c.length ? !1 : ie(t, e, r, n, tt, a);
        }
        if (o === "[object Object]" && (!ve(t) && ve(e) || !me(t) && me(e))) return !1;
        if (qr(t)) {
            if (!qr(e) || Date.prototype.getTime.call(t) !== Date.prototype.getTime.call(e)) return !1;
        } else if (Rr(t)) {
            if (!Rr(e) || !Bo(t, e)) return !1;
        } else if (Fo(t) || t instanceof Error) {
            if (t.message !== e.message || t.name !== e.name) return !1;
        } else if (xo(t)) {
            if (!r && (Do(t) || Uo(t))) {
                if (!Lo(t, e)) return !1;
            } else if (!Vo(t, e)) return !1;
            var u = Ee(t, Se), f = Ee(e, Se);
            return u.length !== f.length ? !1 : ie(t, e, r, n, Ke, u);
        } else {
            if (me(t)) return !me(e) || t.size !== e.size ? !1 : ie(t, e, r, n, $r);
            if (ve(t)) return !ve(e) || t.size !== e.size ? !1 : ie(t, e, r, n, Gr);
            if (No(t)) {
                if (!ko(t, e)) return !1;
            } else if (To(t) && !Wo(t, e)) return !1;
        }
        return ie(t, e, r, n, Ke);
    }
    function Tr(t, e) {
        return e.filter(function(r) {
            return we(t, r);
        });
    }
    function ie(t, e, r, n, o, i) {
        if (arguments.length === 5) {
            i = Object.keys(t);
            var a = Object.keys(e);
            if (i.length !== a.length) return !1;
        }
        for(var c = 0; c < i.length; c++)if (!ae(e, i[c])) return !1;
        if (r && arguments.length === 5) {
            var u = Oe(t);
            if (u.length !== 0) {
                var f = 0;
                for(c = 0; c < u.length; c++){
                    var l = u[c];
                    if (we(t, l)) {
                        if (!we(e, l)) return !1;
                        i.push(l), f++;
                    } else if (we(e, l)) return !1;
                }
                var s = Oe(e);
                if (u.length !== s.length && Tr(e, s).length !== f) return !1;
            } else {
                var p = Oe(e);
                if (p.length !== 0 && Tr(e, p).length !== 0) return !1;
            }
        }
        if (i.length === 0 && (o === Ke || o === tt && t.length === 0 || t.size === 0)) return !0;
        if (n === void 0) n = {
            val1: new Map,
            val2: new Map,
            position: 0
        };
        else {
            var d = n.val1.get(t);
            if (d !== void 0) {
                var v = n.val2.get(e);
                if (v !== void 0) return d === v;
            }
            n.position++;
        }
        n.val1.set(t, n.position), n.val2.set(e, n.position);
        var w = Jo(t, e, r, i, n, o);
        return n.val1.delete(t), n.val2.delete(e), w;
    }
    function _r(t, e, r, n) {
        for(var o = Ae(t), i = 0; i < o.length; i++){
            var a = o[i];
            if (O(e, a, r, n)) return t.delete(a), !0;
        }
        return !1;
    }
    function Br(t) {
        switch(E(t)){
            case "undefined":
                return null;
            case "object":
                return;
            case "symbol":
                return !1;
            case "string":
                t = +t;
            case "number":
                if (Ze(t)) return !1;
        }
        return !0;
    }
    function zo(t, e, r) {
        var n = Br(r);
        return n != null ? n : e.has(n) && !t.has(n);
    }
    function Yo(t, e, r, n, o) {
        var i = Br(r);
        if (i != null) return i;
        var a = e.get(i);
        return a === void 0 && !e.has(i) || !O(n, a, !1, o) ? !1 : !t.has(i) && O(n, a, !1, o);
    }
    function Co(t, e, r, n) {
        for(var o = null, i = Ae(t), a = 0; a < i.length; a++){
            var c = i[a];
            if (E(c) === "object" && c !== null) o === null && (o = new Set), o.add(c);
            else if (!e.has(c)) {
                if (r || !zo(t, e, c)) return !1;
                o === null && (o = new Set), o.add(c);
            }
        }
        if (o !== null) {
            for(var u = Ae(e), f = 0; f < u.length; f++){
                var l = u[f];
                if (E(l) === "object" && l !== null) {
                    if (!_r(o, l, r, n)) return !1;
                } else if (!r && !t.has(l) && !_r(o, l, r, n)) return !1;
            }
            return o.size === 0;
        }
        return !0;
    }
    function Dr(t, e, r, n, o, i) {
        for(var a = Ae(t), c = 0; c < a.length; c++){
            var u = a[c];
            if (O(r, u, o, i) && O(n, e.get(u), o, i)) return t.delete(u), !0;
        }
        return !1;
    }
    function Ho(t, e, r, n) {
        for(var o = null, i = Pr(t), a = 0; a < i.length; a++){
            var c = Ar(i[a], 2), u = c[0], f = c[1];
            if (E(u) === "object" && u !== null) o === null && (o = new Set), o.add(u);
            else {
                var l = e.get(u);
                if (l === void 0 && !e.has(u) || !O(f, l, r, n)) {
                    if (r || !Yo(t, e, u, f, n)) return !1;
                    o === null && (o = new Set), o.add(u);
                }
            }
        }
        if (o !== null) {
            for(var s = Pr(e), p = 0; p < s.length; p++){
                var d = Ar(s[p], 2), u = d[0], v = d[1];
                if (E(u) === "object" && u !== null) {
                    if (!Dr(o, t, u, v, r, n)) return !1;
                } else if (!r && (!t.has(u) || !O(t.get(u), v, !1, n)) && !Dr(o, t, u, v, !1, n)) return !1;
            }
            return o.size === 0;
        }
        return !0;
    }
    function Jo(t, e, r, n, o, i) {
        var a = 0;
        if (i === $r) {
            if (!Co(t, e, r, o)) return !1;
        } else if (i === Gr) {
            if (!Ho(t, e, r, o)) return !1;
        } else if (i === tt) for(; a < t.length; a++)if (ae(t, a)) {
            if (!ae(e, a) || !O(t[a], e[a], r, o)) return !1;
        } else {
            if (ae(e, a)) return !1;
            for(var c = Object.keys(t); a < c.length; a++){
                var u = c[a];
                if (!ae(e, u) || !O(t[u], e[u], r, o)) return !1;
            }
            return c.length === Object.keys(e).length;
        }
        for(a = 0; a < n.length; a++){
            var f = n[a];
            if (!O(t[f], e[f], r, o)) return !1;
        }
        return !0;
    }
    function Xo(t, e) {
        return O(t, e, Go);
    }
    function Qo(t, e) {
        return O(t, e, $o);
    }
    Lr.exports = {
        isDeepEqual: Xo,
        isDeepStrictEqual: Qo
    };
});
var se = g((Ti, on)=>{
    "use strict";
    function N(t) {
        return typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? N = function(r) {
            return typeof r;
        } : N = function(r) {
            return r && typeof Symbol == "function" && r.constructor === Symbol && r !== Symbol.prototype ? "symbol" : typeof r;
        }, N(t);
    }
    function Zo(t, e) {
        if (!(t instanceof e)) throw new TypeError("Cannot call a class as a function");
    }
    var Ko = Fe(), ue = Ko.codes, kr = ue.ERR_AMBIGUOUS_ARGUMENT, ce = ue.ERR_INVALID_ARG_TYPE, ei = ue.ERR_INVALID_ARG_VALUE, ti = ue.ERR_INVALID_RETURN_VALUE, F = ue.ERR_MISSING_ARGS, M = vt(), ri = ii, ni = ri.inspect, Cr = ii.types, oi = Cr.isPromise, rt = Cr.isRegExp, ii1 = Object.assign ? Object.assign : St().assign, Hr = Object.is ? Object.is : Je(), x, Pe;
    function fe() {
        var t = Vr();
        x = t.isDeepEqual, Pe = t.isDeepStrictEqual;
    }
    var Wr = !1, h = on.exports = nt, je = {};
    function j(t) {
        throw t.message instanceof Error ? t.message : new M(t);
    }
    function Jr(t, e, r, n, o) {
        var i = arguments.length, a;
        if (i === 0) a = "Failed";
        else if (i === 1) r = t, t = void 0;
        else {
            if (Wr === !1) {
                Wr = !0;
                var c = __default.emitWarning ? __default.emitWarning : console.warn.bind(console);
                c("assert.fail() with more than one argument is deprecated. Please use assert.strictEqual() instead or only pass a message.", "DeprecationWarning", "DEP0094");
            }
            i === 2 && (n = "!=");
        }
        if (r instanceof Error) throw r;
        var u = {
            actual: t,
            expected: e,
            operator: n === void 0 ? "fail" : n,
            stackStartFn: o || Jr
        };
        r !== void 0 && (u.message = r);
        var f = new M(u);
        throw a && (f.message = a, f.generatedMessage = !0), f;
    }
    h.fail = Jr;
    h.AssertionError = M;
    function Xr(t, e, r, n) {
        if (!r) {
            var o = !1;
            if (e === 0) o = !0, n = "No value argument passed to `assert.ok()`";
            else if (n instanceof Error) throw n;
            var i = new M({
                actual: r,
                expected: !0,
                message: n,
                operator: "==",
                stackStartFn: t
            });
            throw i.generatedMessage = o, i;
        }
    }
    function nt() {
        for(var t = arguments.length, e = new Array(t), r = 0; r < t; r++)e[r] = arguments[r];
        Xr.apply(void 0, [
            nt,
            e.length
        ].concat(e));
    }
    h.ok = nt;
    h.equal = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        e != r && j({
            actual: e,
            expected: r,
            message: n,
            operator: "==",
            stackStartFn: t
        });
    };
    h.notEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        e == r && j({
            actual: e,
            expected: r,
            message: n,
            operator: "!=",
            stackStartFn: t
        });
    };
    h.deepEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        x === void 0 && fe(), x(e, r) || j({
            actual: e,
            expected: r,
            message: n,
            operator: "deepEqual",
            stackStartFn: t
        });
    };
    h.notDeepEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        x === void 0 && fe(), x(e, r) && j({
            actual: e,
            expected: r,
            message: n,
            operator: "notDeepEqual",
            stackStartFn: t
        });
    };
    h.deepStrictEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        x === void 0 && fe(), Pe(e, r) || j({
            actual: e,
            expected: r,
            message: n,
            operator: "deepStrictEqual",
            stackStartFn: t
        });
    };
    h.notDeepStrictEqual = Qr;
    function Qr(t, e, r) {
        if (arguments.length < 2) throw new F("actual", "expected");
        x === void 0 && fe(), Pe(t, e) && j({
            actual: t,
            expected: e,
            message: r,
            operator: "notDeepStrictEqual",
            stackStartFn: Qr
        });
    }
    h.strictEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        Hr(e, r) || j({
            actual: e,
            expected: r,
            message: n,
            operator: "strictEqual",
            stackStartFn: t
        });
    };
    h.notStrictEqual = function t(e, r, n) {
        if (arguments.length < 2) throw new F("actual", "expected");
        Hr(e, r) && j({
            actual: e,
            expected: r,
            message: n,
            operator: "notStrictEqual",
            stackStartFn: t
        });
    };
    var zr = function t(e, r, n) {
        var o = this;
        Zo(this, t), r.forEach(function(i) {
            i in e && (n !== void 0 && typeof n[i] == "string" && rt(e[i]) && e[i].test(n[i]) ? o[i] = n[i] : o[i] = e[i]);
        });
    };
    function ai(t, e, r, n, o, i) {
        if (!(r in t) || !Pe(t[r], e[r])) {
            if (!n) {
                var a = new zr(t, o), c = new zr(e, o, t), u = new M({
                    actual: a,
                    expected: c,
                    operator: "deepStrictEqual",
                    stackStartFn: i
                });
                throw u.actual = t, u.expected = e, u.operator = i.name, u;
            }
            j({
                actual: t,
                expected: e,
                message: n,
                operator: i.name,
                stackStartFn: i
            });
        }
    }
    function Zr(t, e, r, n) {
        if (typeof e != "function") {
            if (rt(e)) return e.test(t);
            if (arguments.length === 2) throw new ce("expected", [
                "Function",
                "RegExp"
            ], e);
            if (N(t) !== "object" || t === null) {
                var o = new M({
                    actual: t,
                    expected: e,
                    message: r,
                    operator: "deepStrictEqual",
                    stackStartFn: n
                });
                throw o.operator = n.name, o;
            }
            var i = Object.keys(e);
            if (e instanceof Error) i.push("name", "message");
            else if (i.length === 0) throw new ei("error", e, "may not be an empty object");
            return x === void 0 && fe(), i.forEach(function(a) {
                typeof t[a] == "string" && rt(e[a]) && e[a].test(t[a]) || ai(t, e, a, r, i, n);
            }), !0;
        }
        return e.prototype !== void 0 && t instanceof e ? !0 : Error.isPrototypeOf(e) ? !1 : e.call({}, t) === !0;
    }
    function Kr(t) {
        if (typeof t != "function") throw new ce("fn", "Function", t);
        try {
            t();
        } catch (e) {
            return e;
        }
        return je;
    }
    function Yr(t) {
        return oi(t) || t !== null && N(t) === "object" && typeof t.then == "function" && typeof t.catch == "function";
    }
    function en(t) {
        return Promise.resolve().then(function() {
            var e;
            if (typeof t == "function") {
                if (e = t(), !Yr(e)) throw new ti("instance of Promise", "promiseFn", e);
            } else if (Yr(t)) e = t;
            else throw new ce("promiseFn", [
                "Function",
                "Promise"
            ], t);
            return Promise.resolve().then(function() {
                return e;
            }).then(function() {
                return je;
            }).catch(function(r) {
                return r;
            });
        });
    }
    function tn(t, e, r, n) {
        if (typeof r == "string") {
            if (arguments.length === 4) throw new ce("error", [
                "Object",
                "Error",
                "Function",
                "RegExp"
            ], r);
            if (N(e) === "object" && e !== null) {
                if (e.message === r) throw new kr("error/message", 'The error message "'.concat(e.message, '" is identical to the message.'));
            } else if (e === r) throw new kr("error/message", 'The error "'.concat(e, '" is identical to the message.'));
            n = r, r = void 0;
        } else if (r != null && N(r) !== "object" && typeof r != "function") throw new ce("error", [
            "Object",
            "Error",
            "Function",
            "RegExp"
        ], r);
        if (e === je) {
            var o = "";
            r && r.name && (o += " (".concat(r.name, ")")), o += n ? ": ".concat(n) : ".";
            var i = t.name === "rejects" ? "rejection" : "exception";
            j({
                actual: void 0,
                expected: r,
                operator: t.name,
                message: "Missing expected ".concat(i).concat(o),
                stackStartFn: t
            });
        }
        if (r && !Zr(e, r, n, t)) throw e;
    }
    function rn(t, e, r, n) {
        if (e !== je) {
            if (typeof r == "string" && (n = r, r = void 0), !r || Zr(e, r)) {
                var o = n ? ": ".concat(n) : ".", i = t.name === "doesNotReject" ? "rejection" : "exception";
                j({
                    actual: e,
                    expected: r,
                    operator: t.name,
                    message: "Got unwanted ".concat(i).concat(o, `
`) + 'Actual message: "'.concat(e && e.message, '"'),
                    stackStartFn: t
                });
            }
            throw e;
        }
    }
    h.throws = function t(e) {
        for(var r = arguments.length, n = new Array(r > 1 ? r - 1 : 0), o = 1; o < r; o++)n[o - 1] = arguments[o];
        tn.apply(void 0, [
            t,
            Kr(e)
        ].concat(n));
    };
    h.rejects = function t(e) {
        for(var r = arguments.length, n = new Array(r > 1 ? r - 1 : 0), o = 1; o < r; o++)n[o - 1] = arguments[o];
        return en(e).then(function(i) {
            return tn.apply(void 0, [
                t,
                i
            ].concat(n));
        });
    };
    h.doesNotThrow = function t(e) {
        for(var r = arguments.length, n = new Array(r > 1 ? r - 1 : 0), o = 1; o < r; o++)n[o - 1] = arguments[o];
        rn.apply(void 0, [
            t,
            Kr(e)
        ].concat(n));
    };
    h.doesNotReject = function t(e) {
        for(var r = arguments.length, n = new Array(r > 1 ? r - 1 : 0), o = 1; o < r; o++)n[o - 1] = arguments[o];
        return en(e).then(function(i) {
            return rn.apply(void 0, [
                t,
                i
            ].concat(n));
        });
    };
    h.ifError = function t(e) {
        if (e != null) {
            var r = "ifError got unwanted exception: ";
            N(e) === "object" && typeof e.message == "string" ? e.message.length === 0 && e.constructor ? r += e.constructor.name : r += e.message : r += ni(e);
            var n = new M({
                actual: e,
                expected: null,
                operator: "ifError",
                message: r,
                stackStartFn: t
            }), o = e.stack;
            if (typeof o == "string") {
                var i = o.split(`
`);
                i.shift();
                for(var a = n.stack.split(`
`), c = 0; c < i.length; c++){
                    var u = a.indexOf(i[c]);
                    if (u !== -1) {
                        a = a.slice(0, u);
                        break;
                    }
                }
                n.stack = "".concat(a.join(`
`), `
`).concat(i.join(`
`));
            }
            throw n;
        }
    };
    function nn() {
        for(var t = arguments.length, e = new Array(t), r = 0; r < t; r++)e[r] = arguments[r];
        Xr.apply(void 0, [
            nn,
            e.length
        ].concat(e));
    }
    h.strict = ii1(nn, h, {
        equal: h.strictEqual,
        deepEqual: h.deepStrictEqual,
        notEqual: h.notStrictEqual,
        notDeepEqual: h.notDeepStrictEqual
    });
    h.strict.strict = h.strict;
});
var un = hn(se());
var { fail: _i , AssertionError: Di , ok: Ui , equal: Mi , notEqual: $i , deepEqual: Gi , notDeepEqual: Bi , deepStrictEqual: Li , notDeepStrictEqual: Vi , strictEqual: ki , notStrictEqual: Wi , rejects: zi , doesNotThrow: Yi , doesNotReject: Ci , ifError: Hi , strict: Ji  } = un, an = un, { default: cn  } = an, ci = st1(an, [
    "default"
]), Xi = cn !== void 0 ? cn : ci;
export { Di as AssertionError, Gi as deepEqual, Li as deepStrictEqual, Ci as doesNotReject, Yi as doesNotThrow, Mi as equal, _i as fail, Hi as ifError, Bi as notDeepEqual, Vi as notDeepStrictEqual, $i as notEqual, Wi as notStrictEqual, Ui as ok, zi as rejects, Ji as strict, ki as strictEqual };
export { Xi as default };
