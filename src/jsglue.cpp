/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#define __STDC_LIMIT_MACROS
#include <stdint.h>

#include "js-config.h"

#ifdef JS_DEBUG
// A hack for MFBT. Guard objects need this to work.
#define DEBUG 1
#endif

#include "jsapi.h"
#include "jsfriendapi.h"
#include "js/Proxy.h"
#include "js/Class.h"
#include "jswrapper.h"

#include "assert.h"

struct ProxyTraps {
    bool (*enter)(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                  js::BaseProxyHandler::Action action, bool *bp);

    bool (*getOwnPropertyDescriptor)(JSContext *cx, JS::HandleObject proxy,
                                     JS::HandleId id,
                                     JS::MutableHandle<JSPropertyDescriptor> desc);
    bool (*defineProperty)(JSContext *cx, JS::HandleObject proxy,
                           JS::HandleId id,
                           JS::Handle<JSPropertyDescriptor> desc,
                           JS::ObjectOpResult &result);
    bool (*ownPropertyKeys)(JSContext *cx, JS::HandleObject proxy,
                            JS::AutoIdVector &props);
    bool (*delete_)(JSContext *cx, JS::HandleObject proxy,
                    JS::HandleId id, JS::ObjectOpResult &result);

    bool (*enumerate)(JSContext *cx, JS::HandleObject proxy,
                      JS::MutableHandleObject objp);

    // getPrototype
    // setPrototype
    // setImmutablePrototype

    bool (*preventExtensions)(JSContext *cx, JS::HandleObject proxy,
                              JS::ObjectOpResult &result);

    bool (*isExtensible)(JSContext *cx, JS::HandleObject proxy, bool *succeeded);

    bool (*has)(JSContext *cx, JS::HandleObject proxy,
                JS::HandleId id, bool *bp);
    bool (*get)(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                JS::HandleId id, JS::MutableHandleValue vp);
    bool (*set)(JSContext *cx, JS::HandleObject proxy, JS::HandleObject receiver,
                JS::HandleId id, JS::MutableHandleValue vp,
                JS::ObjectOpResult &result);

    bool (*call)(JSContext *cx, JS::HandleObject proxy,
                 const JS::CallArgs &args);
    bool (*construct)(JSContext *cx, JS::HandleObject proxy,
                      const JS::CallArgs &args);

    bool (*getPropertyDescriptor)(JSContext *cx, JS::HandleObject proxy,
                                  JS::HandleId id,
                                  JS::MutableHandle<JSPropertyDescriptor> desc);
    bool (*hasOwn)(JSContext *cx, JS::HandleObject proxy,
                   JS::HandleId id, bool *bp);
    bool (*getOwnEnumerablePropertyKeys)(JSContext *cx, JS::HandleObject proxy,
                                         JS::AutoIdVector &props);
    bool (*nativeCall)(JSContext *cx, JS::IsAcceptableThis test,
                       JS::NativeImpl impl, JS::CallArgs args);
    bool (*hasInstance)(JSContext *cx, JS::HandleObject proxy,
                        JS::MutableHandleValue v, bool *bp);
    bool (*objectClassIs)(JS::HandleObject obj, js::ESClassValue classValue,
                          JSContext *cx);
    const char *(*className)(JSContext *cx, JS::HandleObject proxy);
    JSString *(*fun_toString)(JSContext *cx, JS::HandleObject proxy,
                              unsigned indent);
    //bool (*regexp_toShared)(JSContext *cx, JS::HandleObject proxy, RegExpGuard *g);
    bool (*boxedValue_unbox)(JSContext *cx, JS::HandleObject proxy,
                             JS::MutableHandleValue vp);
    bool (*defaultValue)(JSContext *cx, JS::HandleObject obj, JSType hint, JS::MutableHandleValue vp);
    void (*trace)(JSTracer *trc, JSObject *proxy);
    void (*finalize)(JSFreeOp *fop, JSObject *proxy);
    void (*objectMoved)(JSObject *proxy, const JSObject *old);

    bool (*isCallable)(JSObject *obj);
    bool (*isConstructor)(JSObject *obj);

    // watch
    // unwatch
    // getElements

    // weakmapKeyDelegate
    // isScripted
};

static int HandlerFamily;

#define DEFER_TO_TRAP_OR_BASE_CLASS(_base)                                      \
                                                                                \
    virtual bool enter(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,  \
                       _base::Action action, bool *bp) const override           \
    {                                                                           \
        return mTraps.enter                                                     \
               ? mTraps.enter(cx, proxy, id, action, bp)                        \
               : _base::enter(cx, proxy, id, action, bp);                       \
    }                                                                           \
                                                                                \
    /* Standard internal methods. */                                            \
    virtual bool enumerate(JSContext *cx, JS::HandleObject proxy,               \
                           JS::MutableHandleObject objp) const override         \
    {                                                                           \
        return mTraps.enumerate                                                 \
               ? mTraps.enumerate(cx, proxy, objp)                              \
               : _base::enumerate(cx, proxy, objp);                             \
    }                                                                           \
                                                                                \
    virtual bool has(JSContext* cx, JS::HandleObject proxy,                     \
                     JS::HandleId id, bool *bp) const override                  \
    {                                                                           \
        return mTraps.has                                                       \
               ? mTraps.has(cx, proxy, id, bp)                                  \
               : _base::has(cx, proxy, id, bp);                                 \
    }                                                                           \
                                                                                \
    virtual bool get(JSContext* cx, JS::HandleObject proxy,                     \
                     JS::HandleObject receiver,                                 \
                     JS::HandleId id, JS::MutableHandleValue vp) const override \
    {                                                                           \
        return mTraps.get                                                       \
               ? mTraps.get(cx, proxy, receiver, id, vp)                        \
               : _base::get(cx, proxy, receiver, id, vp);                       \
    }                                                                           \
                                                                                \
    virtual bool set(JSContext* cx, JS::HandleObject proxy,                     \
                     JS::HandleObject receiver,                                 \
                     JS::HandleId id, JS::MutableHandleValue vp,                \
                     JS::ObjectOpResult &result) const override                 \
    {                                                                           \
        return mTraps.set                                                       \
               ? mTraps.set(cx, proxy, receiver, id, vp, result)                \
               : _base::set(cx, proxy, receiver, id, vp, result);               \
    }                                                                           \
                                                                                \
    virtual bool call(JSContext* cx, JS::HandleObject proxy,                    \
                      const JS::CallArgs &args) const override                  \
    {                                                                           \
        return mTraps.call                                                      \
               ? mTraps.call(cx, proxy, args)                                   \
               : _base::call(cx, proxy, args);                                  \
    }                                                                           \
                                                                                \
    virtual bool construct(JSContext* cx, JS::HandleObject proxy,               \
                           const JS::CallArgs &args) const override             \
    {                                                                           \
        return mTraps.construct                                                 \
               ? mTraps.construct(cx, proxy, args)                              \
               : _base::construct(cx, proxy, args);                             \
    }                                                                           \
                                                                                \
    /* Spidermonkey extensions. */                                              \
    virtual bool hasOwn(JSContext* cx, JS::HandleObject proxy, JS::HandleId id, \
                        bool* bp) const override                                \
    {                                                                           \
        return mTraps.hasOwn                                                    \
               ? mTraps.hasOwn(cx, proxy, id, bp)                               \
               : _base::hasOwn(cx, proxy, id, bp);                              \
    }                                                                           \
                                                                                \
    virtual bool getOwnEnumerablePropertyKeys(JSContext* cx,                    \
                                              JS::HandleObject proxy,           \
                                              JS::AutoIdVector &props) const override \
    {                                                                           \
        return mTraps.getOwnEnumerablePropertyKeys                              \
               ? mTraps.getOwnEnumerablePropertyKeys(cx, proxy, props)          \
               : _base::getOwnEnumerablePropertyKeys(cx, proxy, props);         \
    }                                                                           \
                                                                                \
    virtual bool nativeCall(JSContext* cx, JS::IsAcceptableThis test,           \
                            JS::NativeImpl impl,                                \
                            JS::CallArgs args) const override                   \
    {                                                                           \
        return mTraps.nativeCall                                                \
               ? mTraps.nativeCall(cx, test, impl, args)                        \
               : _base::nativeCall(cx, test, impl, args);                       \
    }                                                                           \
                                                                                \
    virtual bool hasInstance(JSContext* cx, JS::HandleObject proxy,             \
                             JS::MutableHandleValue v, bool* bp) const override \
    {                                                                           \
        return mTraps.hasInstance                                               \
               ? mTraps.hasInstance(cx, proxy, v, bp)                           \
               : _base::hasInstance(cx, proxy, v, bp);                          \
    }                                                                           \
                                                                                \
    virtual bool objectClassIs(JS::HandleObject obj,                            \
                               js::ESClassValue classValue,                     \
                               JSContext* cx) const override                    \
    {                                                                           \
        return mTraps.objectClassIs                                             \
               ? mTraps.objectClassIs(obj, classValue, cx)                      \
               : _base::objectClassIs(obj, classValue, cx);                     \
    }                                                                           \
                                                                                \
    virtual const char *className(JSContext *cx, JS::HandleObject proxy) const override\
    {                                                                           \
        return mTraps.className                                                 \
               ? mTraps.className(cx, proxy)                                    \
               : _base::className(cx, proxy);                                   \
    }                                                                           \
                                                                                \
    virtual JSString* fun_toString(JSContext* cx, JS::HandleObject proxy,       \
                                   unsigned indent) const override              \
    {                                                                           \
        return mTraps.fun_toString                                              \
               ? mTraps.fun_toString(cx, proxy, indent)                         \
               : _base::fun_toString(cx, proxy, indent);                        \
    }                                                                           \
                                                                                \
    virtual bool boxedValue_unbox(JSContext* cx, JS::HandleObject proxy,        \
                                  JS::MutableHandleValue vp) const override     \
    {                                                                           \
        return mTraps.boxedValue_unbox                                          \
               ? mTraps.boxedValue_unbox(cx, proxy, vp)                         \
               : _base::boxedValue_unbox(cx, proxy, vp);                        \
    }                                                                           \
                                                                                \
    virtual bool defaultValue(JSContext* cx, JS::HandleObject obj, JSType hint, \
                              JS::MutableHandleValue vp) const override         \
    {                                                                           \
        return mTraps.defaultValue                                              \
               ? mTraps.defaultValue(cx, obj, hint, vp)                         \
               : _base::defaultValue(cx, obj, hint, vp);                        \
    }                                                                           \
                                                                                \
    virtual void trace(JSTracer* trc, JSObject* proxy) const override           \
    {                                                                           \
        mTraps.trace                                                            \
        ? mTraps.trace(trc, proxy)                                              \
        : _base::trace(trc, proxy);                                             \
    }                                                                           \
                                                                                \
    virtual void finalize(JSFreeOp* fop, JSObject* proxy) const override        \
    {                                                                           \
        mTraps.finalize                                                         \
        ? mTraps.finalize(fop, proxy)                                           \
        : _base::finalize(fop, proxy);                                          \
    }                                                                           \
                                                                                \
    virtual void objectMoved(JSObject* proxy,                                   \
                             const JSObject *old) const override                \
    {                                                                           \
        mTraps.objectMoved                                                      \
        ? mTraps.objectMoved(proxy, old)                                        \
        : _base::objectMoved(proxy, old);                                       \
    }                                                                           \
                                                                                \
    virtual bool isCallable(JSObject* obj) const override                       \
    {                                                                           \
        return mTraps.isCallable                                                \
               ? mTraps.isCallable(obj)                                         \
               : _base::isCallable(obj);                                        \
    }                                                                           \
                                                                                \
    virtual bool isConstructor(JSObject* obj) const override                    \
    {                                                                           \
        return mTraps.isConstructor                                             \
               ? mTraps.isConstructor(obj)                                      \
               : _base::isConstructor(obj);                                     \
    }

class WrapperProxyHandler : public js::Wrapper
{
    ProxyTraps mTraps;
  public:
    WrapperProxyHandler(const ProxyTraps& aTraps)
    : js::Wrapper(0), mTraps(aTraps) {}

    DEFER_TO_TRAP_OR_BASE_CLASS(js::Wrapper)

    virtual bool getOwnPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                          JS::HandleId id,
                                          JS::MutableHandle<JSPropertyDescriptor> desc) const override
    {
        return mTraps.getOwnPropertyDescriptor
               ? mTraps.getOwnPropertyDescriptor(cx, proxy, id, desc)
               : js::Wrapper::getOwnPropertyDescriptor(cx, proxy, id, desc);
    }

    virtual bool defineProperty(JSContext *cx,
                                JS::HandleObject proxy, JS::HandleId id,
                                JS::MutableHandle<JSPropertyDescriptor> desc,
                                JS::ObjectOpResult &result) const override
    {
        return mTraps.defineProperty
               ? mTraps.defineProperty(cx, proxy, id, desc, result)
               : js::Wrapper::defineProperty(cx, proxy, id, desc, result);
    }

    virtual bool ownPropertyKeys(JSContext *cx, JS::HandleObject proxy,
                                 JS::AutoIdVector &props) const override
    {
        return mTraps.ownPropertyKeys
               ? mTraps.ownPropertyKeys(cx, proxy, props)
               : js::Wrapper::ownPropertyKeys(cx, proxy, props);
    }

    virtual bool delete_(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                         JS::ObjectOpResult &result) const override
    {
        return mTraps.delete_
               ? mTraps.delete_(cx, proxy, id, result)
               : js::Wrapper::delete_(cx, proxy, id, result);
    }

    virtual bool preventExtensions(JSContext *cx, JS::HandleObject proxy,
                                   JS::ObjectOpResult &result) const override
    {
        return mTraps.preventExtensions
               ? mTraps.preventExtensions(cx, proxy, result)
               : js::Wrapper::preventExtensions(cx, proxy, result);
    }

    virtual bool isExtensible(JSContext *cx, JS::HandleObject proxy,
                              bool *succeeded) const override
    {
        return mTraps.isExtensible
               ? mTraps.isExtensible(cx, proxy, succeeded)
               : js::Wrapper::isExtensible(cx, proxy, succeeded);
    }

    virtual bool getPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                       JS::HandleId id,
                                       JS::MutableHandle<JSPropertyDescriptor> desc) const override
    {
        return mTraps.getPropertyDescriptor
               ? mTraps.getPropertyDescriptor(cx, proxy, id, desc)
               : js::Wrapper::getPropertyDescriptor(cx, proxy, id, desc);
    }
};

class ForwardingProxyHandler : public js::BaseProxyHandler
{
    ProxyTraps mTraps;
    const void* mExtra;
  public:
    ForwardingProxyHandler(const ProxyTraps& aTraps, const void* aExtra)
    : js::BaseProxyHandler(&HandlerFamily), mTraps(aTraps), mExtra(aExtra) {}

    const void* getExtra() const {
        return mExtra;
    }

    DEFER_TO_TRAP_OR_BASE_CLASS(BaseProxyHandler)

    virtual bool getOwnPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                          JS::HandleId id,
                                          JS::MutableHandle<JSPropertyDescriptor> desc) const override
    {
        return mTraps.getOwnPropertyDescriptor(cx, proxy, id, desc);
    }

    virtual bool defineProperty(JSContext *cx,
                                JS::HandleObject proxy, JS::HandleId id,
                                JS::MutableHandle<JSPropertyDescriptor> desc,
                                JS::ObjectOpResult &result) const override
    {
        return mTraps.defineProperty(cx, proxy, id, desc, result);
    }

    virtual bool ownPropertyKeys(JSContext *cx, JS::HandleObject proxy,
                                 JS::AutoIdVector &props) const override
    {
        return mTraps.ownPropertyKeys(cx, proxy, props);
    }

    virtual bool delete_(JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
                         JS::ObjectOpResult &result) const override
    {
        return mTraps.delete_(cx, proxy, id, result);
    }

    virtual bool preventExtensions(JSContext *cx, JS::HandleObject proxy,
                                   JS::ObjectOpResult &result) const override
    {
        return mTraps.preventExtensions(cx, proxy, result);
    }

    virtual bool isExtensible(JSContext *cx, JS::HandleObject proxy,
                              bool *succeeded) const override
    {
        return mTraps.isExtensible(cx, proxy, succeeded);
    }

    virtual bool getPropertyDescriptor(JSContext *cx, JS::HandleObject proxy,
                                       JS::HandleId id,
                                       JS::MutableHandle<JSPropertyDescriptor> desc) const override
    {
        return mTraps.getPropertyDescriptor(cx, proxy, id, desc);
    }
};

extern "C" {

bool
InvokeGetOwnPropertyDescriptor(
        const void *handler,
        JSContext *cx, JS::HandleObject proxy, JS::HandleId id,
        JS::MutableHandle<JSPropertyDescriptor> desc)
{
    return static_cast<const ForwardingProxyHandler*>(handler)->
        getOwnPropertyDescriptor(cx, proxy, id, desc);
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
CallJitGetterOp(const JSJitInfo* info, JSContext* cx,
                JS::HandleObject thisObj, void* specializedThis,
                unsigned argc, JS::Value* vp)
{
    JS::CallArgs args = JS::CallArgsFromVp(argc, vp);
    return info->getter(cx, thisObj, specializedThis, JSJitGetterCallArgs(args));
}

bool
CallJitSetterOp(const JSJitInfo* info, JSContext* cx,
                JS::HandleObject thisObj, void* specializedThis,
                unsigned argc, JS::Value* vp)
{
    JS::CallArgs args = JS::CallArgsFromVp(argc, vp);
    return info->setter(cx, thisObj, specializedThis, JSJitSetterCallArgs(args));
}

bool
CallJitMethodOp(const JSJitInfo* info, JSContext* cx,
                JS::HandleObject thisObj, void* specializedThis,
                uint32_t argc, JS::Value* vp)
{
    JS::CallArgs args = JS::CallArgsFromVp(argc, vp);
    return info->method(cx, thisObj, specializedThis, JSJitMethodCallArgs(args));
}

const void*
CreateProxyHandler(const ProxyTraps* aTraps, const void* aExtra)
{
    return new ForwardingProxyHandler(*aTraps, aExtra);
}

const void*
CreateWrapperProxyHandler(const ProxyTraps* aTraps)
{
    return new WrapperProxyHandler(*aTraps);
}

const void*
GetCrossCompartmentWrapper()
{
    return &js::CrossCompartmentWrapper::singleton;
}

JS::ReadOnlyCompileOptions*
NewCompileOptions(JSContext* aCx, const char* aFile, unsigned aLine)
{
    JS::OwningCompileOptions *opts = new JS::OwningCompileOptions(aCx);
    opts->setFileAndLine(aCx, aFile, aLine);
    opts->setVersion(JSVERSION_DEFAULT);
    return opts;
}

void
DeleteCompileOptions(JS::ReadOnlyCompileOptions *aOpts)
{
    delete static_cast<JS::OwningCompileOptions *>(aOpts);
}

JSObject*
NewProxyObject(JSContext* aCx, const void* aHandler, JS::HandleValue aPriv,
               JSObject* proto, JSObject* parent, JSObject* call,
               JSObject* construct)
{
    js::ProxyOptions options;
    return js::NewProxyObject(aCx, (js::BaseProxyHandler*)aHandler, aPriv, proto,
                              options);
}

JSObject*
WrapperNew(JSContext* aCx, JS::HandleObject aObj, const void* aHandler)
{
    return js::Wrapper::New(aCx, aObj, (const js::Wrapper*)aHandler);
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
RUST_JSID_IS_INT(JS::HandleId id)
{
    return JSID_IS_INT(id);
}

int
RUST_JSID_TO_INT(JS::HandleId id)
{
    return JSID_TO_INT(id);
}

bool
RUST_JSID_IS_STRING(JS::HandleId id)
{
    return JSID_IS_STRING(id);
}

JSString*
RUST_JSID_TO_STRING(JS::HandleId id)
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

const JSErrorFormatString*
RUST_js_GetErrorMessage(void* userRef, uint32_t errorNumber)
{
    return js::GetErrorMessage(userRef, errorNumber);
}

bool
IsProxyHandlerFamily(JSObject* obj)
{
    return js::GetProxyHandler(obj)->family() == &HandlerFamily;
}

const void*
GetProxyHandlerExtra(JSObject* obj)
{
    const js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
    assert(handler->family() == &HandlerFamily);
    return static_cast<const ForwardingProxyHandler*>(handler)->getExtra();
}

const void*
GetProxyHandler(JSObject* obj)
{
    const js::BaseProxyHandler* handler = js::GetProxyHandler(obj);
    assert(handler->family() == &HandlerFamily);
    return handler;
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

bool
AppendToAutoIdVector(JS::AutoIdVector* v, jsid id)
{
    return v->append(id);
}

JS::AutoObjectVector*
CreateAutoObjectVector(JSContext* aCx)
{
    JS::AutoObjectVector* vec = new JS::AutoObjectVector(aCx);
    return vec;
}

bool
AppendToAutoObjectVector(JS::AutoObjectVector* v, JSObject* obj)
{
    return v->append(obj);
}

void
DeleteAutoObjectVector(JS::AutoObjectVector* v)
{
    delete v;
}

} // extern "C"
