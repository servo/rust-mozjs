/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#define __STDC_LIMIT_MACROS
#include <stdint.h>
#include "jsapi.h"
#include "jsfriendapi.h"
#include "jsproxy.h"
#include "jswrapper.h"

#include "assert.h"

struct ProxyTraps {
    bool (*preventExtensions)(JSContext *cx, JS::HandleObject proxy);
    bool (*getPropertyDescriptor)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                                  JS::MutableHandle<JSPropertyDescriptor> desc,
                                  unsigned flags);
    bool (*getOwnPropertyDescriptor)(JSContext *cx, JS::HandleObject proxy,
                                     JS::HandleId id,
                                     JS::MutableHandle<JSPropertyDescriptor> desc,
                                     unsigned flags);
    bool (*defineProperty)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                           JS::MutableHandle<JSPropertyDescriptor> desc);
    bool (*getOwnPropertyNames)(JSContext *cx, JS::HandleObject proxy,
                                JS::AutoIdVector &props);
    bool (*delete_)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp);
    bool (*enumerate)(JSContext *cx, JS::HandleObject proxy, JS::AutoIdVector &props);

    bool (*has)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp);
    bool (*hasOwn)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp);
    bool (*get)(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                JS::HandleId id, JS::MutableHandleValue vp);
    bool (*set)(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                JS::HandleId id, bool strict, JS::MutableHandleValue vp);
    bool (*keys)(JSContext *cx, JS::HandleObject proxy, JS::AutoIdVector &props);
    bool (*iterate)(JSContext *cx, JS::HandleObject proxy, unsigned flags,
                    JS::MutableHandleValue vp);

    bool (*isExtensible)(JSContext *cx, JS::HandleObject proxy, bool *extensible);
    bool (*call)(JSContext *cx, JS::HandleObject proxy, unsigned argc, JS::MutableHandleValue vp);
    bool (*construct)(JSContext *cx, JS::HandleObject proxy, unsigned argc, JS::MutableHandleValue argv, JS::MutableHandleValue rval);
    bool (*nativeCall)(JSContext *cx, JS::IsAcceptableThis test, JS::NativeImpl impl, JS::CallArgs args);
    bool (*hasInstance)(JSContext *cx, JS::HandleObject proxy, const JS::MutableHandleValue vp, bool *bp);
    bool (*objectClassIs)(JS::HandleObject obj, js::ESClassValue classValue, JSContext *cx);
    JSString *(*fun_toString)(JSContext *cx, JS::HandleObject proxy, unsigned indent);
    //bool (*regexp_toShared)(JSContext *cx, JS::HandleObject proxy, RegExpGuard *g);
    bool (*defaultValue)(JSContext *cx, JS::HandleObject obj, JSType hint, JS::MutableHandleValue vp);
    void (*finalize)(JSFreeOp *fop, JSObject* proxy);
    bool (*getPrototypeOf)(JSContext *cx, JS::HandleObject proxy, JS::MutableHandleObject proto);
    void (*trace)(JSTracer *trc, JS::HandleObject proxy);
};

int HandlerFamily = 0 /*JSPROXYSLOT_EXPANDO*/;

class WrapperProxyHandler : public js::DirectProxyHandler
{
    ProxyTraps mTraps;
  public:
    WrapperProxyHandler(const ProxyTraps& aTraps)
    : js::DirectProxyHandler(0), mTraps(aTraps) {}

    virtual bool isOuterWindow() {
        return true;
    }

    virtual bool preventExtensions(JSContext *cx, JS::HandleObject proxy)
    {
        return mTraps.preventExtensions ?
                mTraps.preventExtensions(cx, proxy) :
                DirectProxyHandler::preventExtensions(cx, proxy);
    }

    virtual bool getPropertyDescriptor(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                                       JS::MutableHandle<JSPropertyDescriptor> desc, unsigned flags)
    {
        return mTraps.getPropertyDescriptor ?
                mTraps.getPropertyDescriptor(cx, proxy, id, desc, flags) :
                DirectProxyHandler::getPropertyDescriptor(cx, proxy, id, desc, flags);
    }

    virtual bool getOwnPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                          JS::HandleId id, JS::MutableHandle<JSPropertyDescriptor> desc,
                                          unsigned flags)
    {
        return mTraps.getOwnPropertyDescriptor ?
                mTraps.getOwnPropertyDescriptor(cx, proxy, id, desc, flags) :
                DirectProxyHandler::getOwnPropertyDescriptor(cx, proxy, id, desc, flags);
    }

    virtual bool defineProperty(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                                JS::MutableHandle<JSPropertyDescriptor> desc)
    {
        return mTraps.defineProperty ?
                mTraps.defineProperty(cx, proxy, id, desc) :
                DirectProxyHandler::defineProperty(cx, proxy, id, desc);
    }

    virtual bool getOwnPropertyNames(JSContext *cx, JS::HandleObject proxy,
                                     JS::AutoIdVector &props)
    {
        return mTraps.getOwnPropertyNames ?
                mTraps.getOwnPropertyNames(cx, proxy, props) :
                DirectProxyHandler::getOwnPropertyNames(cx, proxy, props);
    }

    virtual bool delete_(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                         bool *bp)
    {
        return mTraps.delete_ ?
                mTraps.delete_(cx, proxy, id, bp) :
                DirectProxyHandler::delete_(cx, proxy, id, bp);
    }

    virtual bool enumerate(JSContext *cx, JS::HandleObject proxy,
                           JS::AutoIdVector &props)
    {
        return mTraps.enumerate ?
                mTraps.enumerate(cx, proxy, props) :
                DirectProxyHandler::enumerate(cx, proxy, props);
    }

    /* ES5 Harmony derived proxy traps. */
    virtual bool has(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                     bool *bp)
    {
        return mTraps.has ?
                mTraps.has(cx, proxy, id, bp) :
                DirectProxyHandler::has(cx, proxy, id, bp);
    }

    virtual bool hasOwn(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                        bool *bp)
    {
        return mTraps.hasOwn ?
                mTraps.hasOwn(cx, proxy, id, bp) :
                DirectProxyHandler::hasOwn(cx, proxy, id, bp);
    }

    virtual bool get(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                     JS::HandleId id, JS::MutableHandleValue vp)
    {
        return mTraps.get ?
                mTraps.get(cx, proxy, receiver, id, vp) :
                DirectProxyHandler::get(cx, proxy, receiver, id, vp);
    }

    virtual bool set(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                     JS::HandleId id, bool strict, JS::MutableHandleValue vp)
    {
        return mTraps.set ?
                mTraps.set(cx, proxy, receiver, id, strict, vp) :
                DirectProxyHandler::set(cx, proxy, receiver, id, strict, vp);
    }

    virtual bool keys(JSContext *cx, JS::HandleObject proxy,
                      JS::AutoIdVector &props)
    {
        return mTraps.keys ?
                mTraps.keys(cx, proxy, props) :
                DirectProxyHandler::keys(cx, proxy, props);
    }

    virtual bool iterate(JSContext *cx, JS::HandleObject proxy, unsigned flags,
                         JS::MutableHandleValue vp)
    {
        return mTraps.iterate ?
                mTraps.iterate(cx, proxy, flags, vp) :
                DirectProxyHandler::iterate(cx, proxy, flags, vp);
    }

    /* Spidermonkey extensions. */
    virtual bool isExtensible(JSContext *cx, JS::HandleObject proxy, bool *extensible)
    {
        return mTraps.isExtensible ?
                mTraps.isExtensible(cx, proxy, extensible) :
                DirectProxyHandler::isExtensible(cx, proxy, extensible);
    }

    virtual bool call(JSContext *cx, JS::HandleObject proxy, const JS::CallArgs &args)
    {
        return mTraps.call ?
                mTraps.call(cx, proxy, args.length(), args[0]) :
                DirectProxyHandler::call(cx, proxy, args);
    }

    virtual bool construct(JSContext *cx, JS::HandleObject proxy, const JS::CallArgs &args)
    {
        return mTraps.construct ?
                mTraps.construct(cx, proxy, args.length(), args[0], args.rval()) :
                DirectProxyHandler::construct(cx, proxy, args);
    }

    virtual bool nativeCall(JSContext *cx, JS::IsAcceptableThis test, JS::NativeImpl impl, JS::CallArgs args)
    {
        return mTraps.nativeCall ?
                mTraps.nativeCall(cx, test, impl, args) :
                DirectProxyHandler::nativeCall(cx, test, impl, args);
    }

    virtual bool hasInstance(JSContext *cx, JS::HandleObject proxy, JS::MutableHandleValue v,
                             bool *bp)
    {
        return mTraps.hasInstance ?
                mTraps.hasInstance(cx, proxy, v, bp) :
                DirectProxyHandler::hasInstance(cx, proxy, v, bp);
    }

    virtual bool objectClassIs(JS::HandleObject obj, js::ESClassValue classValue, JSContext *cx)
    {
        return mTraps.objectClassIs ?
                mTraps.objectClassIs(obj, classValue, cx) :
                DirectProxyHandler::objectClassIs(obj, classValue, cx);
    }

    virtual JSString *fun_toString(JSContext *cx, JS::HandleObject proxy, unsigned indent)
    {
        return mTraps.fun_toString ?
                mTraps.fun_toString(cx, proxy, indent) :
                DirectProxyHandler::fun_toString(cx, proxy, indent);
    }

    /*virtual bool regexp_toShared(JSContext *cx, JSObject *proxy, RegExpGuard *g)
      {
      return mTraps.regexp_toShared ?
      mTraps.regexp_toShared(cx, proxy, g) :
      DirectProxyHandler::regexp_toShared(cx, proxy, g);
      }*/

    virtual bool defaultValue(JSContext *cx, JS::HandleObject obj, JSType hint, JS::MutableHandleValue vp)
    {
        return mTraps.defaultValue ?
                mTraps.defaultValue(cx, obj, hint, vp) :
                DirectProxyHandler::defaultValue(cx, obj, hint, vp);
    }

    virtual void finalize(JSFreeOp *fop, JSObject *proxy)
    {
        if (mTraps.finalize) {
            mTraps.finalize(fop, proxy);
        } else {
            DirectProxyHandler::finalize(fop, proxy);
        }
    }

    virtual bool getPrototypeOf(JSContext *cx, JS::HandleObject proxy, JS::MutableHandleObject proto)
    {
        return mTraps.getPrototypeOf ?
                mTraps.getPrototypeOf(cx, proxy, proto) :
                DirectProxyHandler::getPrototypeOf(cx, proxy, proto);
    }

    virtual void trace(JSTracer *trc, JS::HandleObject proxy)
    {
        return mTraps.trace ?
                mTraps.trace(trc, proxy) :
                DirectProxyHandler::trace(trc, proxy);
    }
};

class ForwardingProxyHandler : public js::BaseProxyHandler
{
    ProxyTraps mTraps;
    void* mExtra;
  public:
    ForwardingProxyHandler(const ProxyTraps& aTraps, void* aExtra)
    : js::BaseProxyHandler(&HandlerFamily), mTraps(aTraps), mExtra(aExtra) {}

    void* getExtra() {
        return mExtra;
    }

    virtual bool preventExtensions(JSContext *cx, JS::HandleObject proxy)
    {
        return mTraps.preventExtensions(cx, proxy);
    }

    virtual bool getPropertyDescriptor(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                                       JS::MutableHandle<JSPropertyDescriptor> desc,
                                       unsigned flags)
    {
        return mTraps.getPropertyDescriptor(cx, proxy, id, desc, flags);
    }

    virtual bool getOwnPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                          JS::HandleId id,
                                          JS::MutableHandle<JSPropertyDescriptor> desc,
                                          unsigned flags)
    {
        return mTraps.getOwnPropertyDescriptor(cx, proxy, id, desc, flags);
    }

    virtual bool defineProperty(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                                JS::MutableHandle<JSPropertyDescriptor> desc)
    {
        return mTraps.defineProperty(cx, proxy, id, desc);
    }

    virtual bool getOwnPropertyNames(JSContext *cx, JS::HandleObject proxy,
                                     JS::AutoIdVector &props)
    {
        return mTraps.getOwnPropertyNames(cx, proxy, props);
    }

    virtual bool delete_(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp)
    {
        return mTraps.delete_(cx, proxy, id, bp);
    }

    virtual bool enumerate(JSContext *cx, JS::HandleObject proxy,
                           JS::AutoIdVector &props)
    {
        return mTraps.enumerate(cx, proxy, props);
    }

    /* ES5 Harmony derived proxy traps. */
    virtual bool has(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp)
    {
        return mTraps.has ?
               mTraps.has(cx, proxy, id, bp) :
               BaseProxyHandler::has(cx, proxy, id, bp);
    }

    virtual bool hasOwn(JSContext *cx, JS::HandleObject proxy, JS::HandleId id, bool *bp)
    {
        return mTraps.hasOwn ?
               mTraps.hasOwn(cx, proxy, id, bp) :
               BaseProxyHandler::hasOwn(cx, proxy, id, bp);
    }

    virtual bool get(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                     JS::HandleId id, JS::MutableHandleValue vp)
    {
        return mTraps.get ?
               mTraps.get(cx, proxy, receiver, id, vp) :
               BaseProxyHandler::get(cx, proxy, receiver, id, vp);
    }

    virtual bool set(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                     JS::HandleId id, bool strict, JS::MutableHandleValue vp)
    {
        return mTraps.set ?
               mTraps.set(cx, proxy, receiver, id, strict, vp) :
               BaseProxyHandler::set(cx, proxy, receiver, id, strict, vp);
    }

    virtual bool keys(JSContext *cx, JS::HandleObject proxy, JS::AutoIdVector &props)
    {
        return mTraps.keys ?
                mTraps.keys(cx, proxy, props) :
                BaseProxyHandler::keys(cx, proxy, props);
    }

    virtual bool iterate(JSContext *cx, JS::HandleObject proxy, unsigned flags,
                         JS::MutableHandleValue vp)
    {
        return mTraps.iterate ?
                mTraps.iterate(cx, proxy, flags, vp) :
                BaseProxyHandler::iterate(cx, proxy, flags, vp);
    }

    /* Spidermonkey extensions. */
    virtual bool isExtensible(JSContext *cx, JS::HandleObject proxy, bool *extensible)
    {
        return mTraps.isExtensible(cx, proxy, extensible);
    }

    virtual bool call(JSContext *cx, JS::HandleObject proxy, const JS::CallArgs &args)
    {
        return mTraps.call ?
                mTraps.call(cx, proxy, args.length(), args[0]) :
                BaseProxyHandler::call(cx, proxy, args);
    }

    virtual bool construct(JSContext *cx, JS::HandleObject proxy, const JS::CallArgs &args)
    {
        return mTraps.construct ?
                mTraps.construct(cx, proxy, args.length(), args[0], args.rval()) :
                BaseProxyHandler::construct(cx, proxy, args);
    }

    virtual bool nativeCall(JSContext *cx, JS::IsAcceptableThis test, JS::NativeImpl impl, JS::CallArgs args)
    {
        return mTraps.nativeCall ?
                mTraps.nativeCall(cx, test, impl, args) :
                BaseProxyHandler::nativeCall(cx, test, impl, args);
    }

    virtual bool hasInstance(JSContext *cx, JS::HandleObject proxy, const JS::MutableHandleValue vp, bool *bp)
    {
        return mTraps.hasInstance ?
                mTraps.hasInstance(cx, proxy, vp, bp) :
                BaseProxyHandler::hasInstance(cx, proxy, vp, bp);
    }

    virtual bool objectClassIs(JS::HandleObject obj, js::ESClassValue classValue, JSContext *cx)
    {
        return mTraps.objectClassIs ?
                mTraps.objectClassIs(obj, classValue, cx) :
                BaseProxyHandler::objectClassIs(obj, classValue, cx);
    }

    virtual JSString *fun_toString(JSContext *cx, JS::HandleObject proxy, unsigned indent)
    {
        return mTraps.fun_toString ?
                mTraps.fun_toString(cx, proxy, indent) :
                BaseProxyHandler::fun_toString(cx, proxy, indent);
    }

    /*virtual bool regexp_toShared(JSContext *cx, JS::HandleObject proxy, RegExpGuard *g)
    {
        return mTraps.regexp_toShared ?
                mTraps.regexp_toShared(cx, proxy, g) :
                BaseProxyHandler::regexp_toShared(cx, proxy, g);
                }*/

    virtual bool defaultValue(JSContext *cx, JS::HandleObject obj, JSType hint, JS::MutableHandleValue vp)
    {
        return mTraps.defaultValue ?
                mTraps.defaultValue(cx, obj, hint, vp) :
                BaseProxyHandler::defaultValue(cx, obj, hint, vp);
    }

    virtual void finalize(JSFreeOp *fop, JS::HandleObject proxy)
    {
        if (mTraps.finalize) {
            mTraps.finalize(fop, proxy);
        } else {
            BaseProxyHandler::finalize(fop, proxy);
        }
    }

    virtual bool getPrototypeOf(JSContext *cx, JS::HandleObject proxy, JS::MutableHandleObject proto)
    {
        return mTraps.getPrototypeOf ?
                mTraps.getPrototypeOf(cx, proxy, proto) :
                BaseProxyHandler::getPrototypeOf(cx, proxy, proto);
    }

    virtual void trace(JSTracer *trc, JS::HandleObject proxy)
    {
        return mTraps.trace ?
                mTraps.trace(trc, proxy) :
                BaseProxyHandler::trace(trc, proxy);
    }
};

extern "C" {

bool
InvokeGetOwnPropertyDescriptor(
        void* handler,
        JSContext *cx, JS::HandleObject proxy,
        JS::HandleId id, JS::MutableHandle<JSPropertyDescriptor> desc,
        unsigned flags)
{
    return static_cast<ForwardingProxyHandler*>(handler)->getOwnPropertyDescriptor(cx, proxy,
                                                                                   id, desc,
                                                                                   flags);
}

jsval
RUST_JS_NumberValue(double d)
{
    return JS_NumberValue(d);
}

const JSJitInfo*
RUST_FUNCTION_VALUE_TO_JITINFO(jsval v)
{
    return FUNCTION_VALUE_TO_JITINFO(v);
}

bool
CallJitGetterOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, const JS::CallArgs& args)
{
    struct {
        JSObject** obj;
    } tmp = { &thisObj };
    return ((JSJitGetterOp)info->getter)(cx, *reinterpret_cast<JS::HandleObject*>(&tmp), specializedThis, JSJitGetterCallArgs(args));
}

bool
CallJitSetterOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, const JS::CallArgs& args)
{
    struct {
        JSObject** obj;
    } tmp = { &thisObj };
    return ((JSJitSetterOp)info->setter)(cx, *reinterpret_cast<JS::HandleObject*>(&tmp), specializedThis, JSJitSetterCallArgs(args));
}

bool
CallJitMethodOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, uint32_t argc, const JS::CallArgs& args)
{
    struct {
        JSObject** obj;
    } tmp = { &thisObj };
    return ((JSJitMethodOp)info->method)(cx, *reinterpret_cast<JS::HandleObject*>(&tmp), specializedThis, JSJitMethodCallArgs(args));
}

void
SetFunctionNativeReserved(JSObject* fun, size_t which, js::Value* val)
{
    js::SetFunctionNativeReserved(fun, which, *val);
}

const js::Value*
GetFunctionNativeReserved(JSObject* fun, size_t which)
{
    return &js::GetFunctionNativeReserved(fun, which);
}

const void*
CreateProxyHandler(const ProxyTraps* aTraps, void* aExtra)
{
    return new ForwardingProxyHandler(*aTraps, aExtra);
}

const void*
CreateWrapperProxyHandler(const ProxyTraps* aTraps)
{
    return new WrapperProxyHandler(*aTraps);
}

JSObject*
NewProxyObject(JSContext* aCx, void* aHandler, JS::HandleValue priv,
               JSObject* proto, JSObject* parent, JSObject* call,
               JSObject* construct)
{
    js::ProxyOptions options;
    //XXXjdm options.setClass(clasp);
    return js::NewProxyObject(aCx, (js::BaseProxyHandler*)aHandler, priv, proto,
                              parent, options);
}

JSObject*
WrapperNew(JSContext* aCx, JS::HandleObject aObj, JS::HandleObject aParent, void* aHandler)
{
    js::WrapperOptions options;
    JS::RootedObject proto(aCx);
    assert(js::GetObjectProto(aCx, aParent, &proto));
    options.setProto(proto.get());
    return js::Wrapper::New(aCx, aObj, aParent, (js::Wrapper*)aHandler, &options);
}

jsval
GetProxyExtra(JSObject* obj, uint32_t slot)
{
    return js::GetProxyExtra(obj, slot);
}

jsval
GetProxyPrivate(JSObject* obj)
{
    return js::GetProxyPrivate(obj);
}

void
SetProxyExtra(JSObject* obj, uint32_t slot, jsval val)
{
    return js::SetProxyExtra(obj, slot, val);
}

bool
GetObjectProto(JSContext* cx, JS::HandleObject obj, JS::MutableHandleObject proto)
{
    js::GetObjectProto(cx, obj, proto);
}

JSObject*
GetObjectParent(JSObject* obj)
{
    return js::GetObjectParent(obj);
}

bool
RUST_JSID_IS_INT(jsid id)
{
    return JSID_IS_INT(id);
}

int
RUST_JSID_TO_INT(jsid id)
{
    return JSID_TO_INT(id);
}

bool
RUST_JSID_IS_STRING(jsid id)
{
    return JSID_IS_STRING(id);
}

JSString*
RUST_JSID_TO_STRING(jsid id)
{
    return JSID_TO_STRING(id);
}

void
RUST_SET_JITINFO(JSFunction* func, const JSJitInfo* info) {
    SET_JITINFO(func, info);
}

jsid
RUST_INTERNED_STRING_TO_JSID(JSContext* cx, JSString* str) {
    return INTERNED_STRING_TO_JSID(cx, str);
}

JSFunction*
DefineFunctionWithReserved(JSContext* cx, JSObject* obj, char* name, JSNative call,
                           uint32_t nargs, uint32_t attrs)
{
    return js::DefineFunctionWithReserved(cx, obj, name, call, nargs, attrs);
}

const JSClass*
GetObjectJSClass(JSObject* obj)
{
    return js::GetObjectJSClass(obj);
}

const JSErrorFormatString*
RUST_js_GetErrorMessage(void* userRef, char* locale, uint32_t errorNumber)
{
    return js_GetErrorMessage(userRef, locale, errorNumber);
}

bool
IsProxyHandlerFamily(JSObject* obj)
{
    return js::GetProxyHandler(obj)->family() == &HandlerFamily;
}

void*
GetProxyHandlerExtra(JSObject* obj)
{
    js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
    assert(handler->family() == &HandlerFamily);
    return static_cast<ForwardingProxyHandler*>(handler)->getExtra();
}

void*
GetProxyHandler(JSObject* obj)
{
    js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
    assert(handler->family() == &HandlerFamily);
    return handler;
}

JSObject*
GetGlobalForObjectCrossCompartment(JSObject* obj)
{
    return js::GetGlobalForObjectCrossCompartment(obj);
}

void
ReportError(JSContext* aCx, const char* aError)
{
#ifdef DEBUG
    for (const char* p = aError; *p; ++p) {
        assert(*p != '%');
    }
#endif
    JS_ReportError(aCx, aError);
}

bool
IsWrapper(JSObject* obj)
{
    return js::IsWrapper(obj);
}

JSObject*
UnwrapObject(JSObject* obj, bool stopAtOuter)
{
    return js::CheckedUnwrap(obj, stopAtOuter);
}

void
ContextOptions_SetVarObjFix(JSContext* cx, bool enable)
{
    JS::ContextOptionsRef(cx).setVarObjFix(true);
}

void
CompartmentOptions_SetTraceGlobal(JSContext* cx, JSTraceOp op)
{
    JS::CompartmentOptionsRef(cx).setTrace(op);
}

void
CompartmentOptions_SetVersion(JSContext* cx, JSVersion version)
{
    JS::CompartmentOptionsRef(cx).setVersion(version);
}

bool
ToBoolean(JS::HandleValue v)
{
    return JS::ToBoolean(v);
}

JSString*
ToString(JSContext* cx, JS::HandleValue v)
{
    return JS::ToString(cx, v);
}

bool
ToNumber(JSContext* cx, JS::HandleValue v, double* out)
{
    return JS::ToNumber(cx, v, out);
}

bool
ToUint16(JSContext* cx, JS::HandleValue v, uint16_t* out)
{
    return JS::ToUint16(cx, v, out);
}

bool
ToInt32(JSContext* cx, JS::HandleValue v, int32_t* out)
{
    return JS::ToInt32(cx, v, out);
}

bool
ToUint32(JSContext* cx, JS::HandleValue v, uint32_t* out)
{
    return JS::ToUint32(cx, v, out);
}

bool
ToInt64(JSContext* cx, JS::HandleValue v, int64_t* out)
{
    return JS::ToInt64(cx, v, out);
}

bool
ToUint64(JSContext* cx, JS::HandleValue v, uint64_t* out)
{
    return JS::ToUint64(cx, v, out);
}

bool
AddObjectRoot(JSContext* cx, JSObject** obj)
{
    return JS::AddObjectRoot(cx, reinterpret_cast<JS::Heap<JSObject*>*>(obj));
}

void
RemoveObjectRoot(JSContext* cx, JSObject** obj)
{
    JS::RemoveObjectRoot(cx, reinterpret_cast<JS::Heap<JSObject*>*>(obj));
}

JSObject*
NewGlobalObject(JSContext* cx, const JSClass *clasp, JSPrincipals* principals,
                JS::OnNewGlobalHookOption hookOption)
{
    return JS_NewGlobalObject(cx, clasp, principals, hookOption);
}

bool
CallFunctionValue(JSContext* cx, JS::HandleObject obj, JS::HandleValue fval,
                  JS::MutableHandleValue rval)
{
    return JS_CallFunctionValue(cx, obj, fval, JS::HandleValueArray::empty(), rval);
}

} // extern "C"
