//XXXjdm whyyyyyyyyyyy
#define UINT32_MAX ((uint32_t)-1)

#include "jsapi.h"
#include "jsfriendapi.h"
#include "jsproxy.h"
#include "jsclass.h"

enum StubType {
    PROPERTY_STUB,
    STRICT_PROPERTY_STUB,
    ENUMERATE_STUB,
    CONVERT_STUB,
    RESOLVE_STUB,
};

struct ProxyTraps {
    bool (*getPropertyDescriptor)(JSContext *cx, JSObject *proxy, jsid id,
                                  bool set, JSPropertyDescriptor *desc);
    bool (*getOwnPropertyDescriptor)(JSContext *cx, JSObject *proxy,
                                     jsid id, bool set,
                                     JSPropertyDescriptor *desc);
    bool (*defineProperty)(JSContext *cx, JSObject *proxy, jsid id,
                           JSPropertyDescriptor *desc);
    bool (*getOwnPropertyNames)(JSContext *cx, JSObject *proxy,
                                JS::AutoIdVector &props);
    bool (*delete_)(JSContext *cx, JSObject *proxy, jsid id, bool *bp);
    bool (*enumerate)(JSContext *cx, JSObject *proxy,
                      JS::AutoIdVector &props);

    bool (*has)(JSContext *cx, JSObject *proxy, jsid id, bool *bp);
    bool (*hasOwn)(JSContext *cx, JSObject *proxy, jsid id, bool *bp);
    bool (*get)(JSContext *cx, JSObject *proxy, JSObject *receiver,
                jsid id, JS::Value *vp);
    bool (*set)(JSContext *cx, JSObject *proxy, JSObject *receiver,
                jsid id, bool strict, JS::Value *vp);
    bool (*keys)(JSContext *cx, JSObject *proxy, JS::AutoIdVector &props);
    bool (*iterate)(JSContext *cx, JSObject *proxy, unsigned flags,
                    JS::Value *vp);

    bool (*call)(JSContext *cx, JSObject *proxy, unsigned argc, JS::Value *vp);
    bool (*construct)(JSContext *cx, JSObject *proxy, unsigned argc, JS::Value *argv, JS::Value *rval);
    bool (*nativeCall)(JSContext *cx, JS::IsAcceptableThis test, JS::NativeImpl impl, JS::CallArgs args);
    bool (*hasInstance)(JSContext *cx, JSObject *proxy, const JS::Value *vp, bool *bp);
    JSType (*typeOf)(JSContext *cx, JSObject *proxy);
    bool (*objectClassIs)(JSObject *obj, js::ESClassValue classValue, JSContext *cx);
    JSString *(*obj_toString)(JSContext *cx, JSObject *proxy);
    JSString *(*fun_toString)(JSContext *cx, JSObject *proxy, unsigned indent);
    //bool (*regexp_toShared)(JSContext *cx, JSObject *proxy, RegExpGuard *g);
    bool (*defaultValue)(JSContext *cx, JSObject *obj, JSType hint, JS::Value *vp);
    bool (*iteratorNext)(JSContext *cx, JSObject *proxy, JS::Value *vp);
    void (*finalize)(JSFreeOp *fop, JSObject *proxy);
    bool (*getElementIfPresent)(JSContext *cx, JSObject *obj, JSObject *receiver,
                                uint32_t index, JS::Value *vp, bool *present);
    bool (*getPrototypeOf)(JSContext *cx, JSObject *proxy, JSObject **proto);
};

int HandlerFamily = js::JSSLOT_PROXY_EXTRA + 0 /*JSPROXYSLOT_EXPANDO*/;

class ForwardingProxyHandler : public js::BaseProxyHandler
{
    ProxyTraps mTraps;
  public:
    ForwardingProxyHandler(const ProxyTraps& aTraps)
    : js::BaseProxyHandler(&HandlerFamily), mTraps(aTraps) {}

    virtual bool getPropertyDescriptor(JSContext *cx, JSObject *proxy, jsid id,
                                       bool set, JSPropertyDescriptor *desc)
    {
        return mTraps.getPropertyDescriptor(cx, proxy, id, set, desc);
    }

    virtual bool getOwnPropertyDescriptor(JSContext *cx, JSObject *proxy,
                                          jsid id, bool set,
                                          JSPropertyDescriptor *desc)
    {
        return mTraps.getOwnPropertyDescriptor(cx, proxy, id, set, desc);
    }

    virtual bool defineProperty(JSContext *cx, JSObject *proxy, jsid id,
                                JSPropertyDescriptor *desc)
    {
        return mTraps.defineProperty(cx, proxy, id, desc);
    }

    virtual bool getOwnPropertyNames(JSContext *cx, JSObject *proxy,
                                     JS::AutoIdVector &props)
    {
        return mTraps.getOwnPropertyNames(cx, proxy, props);
    }

    virtual bool delete_(JSContext *cx, JSObject *proxy, jsid id, bool *bp)
    {
        return mTraps.delete_(cx, proxy, id, bp);
    }

    virtual bool enumerate(JSContext *cx, JSObject *proxy,
                           JS::AutoIdVector &props)
    {
        return mTraps.enumerate(cx, proxy, props);
    }

    /* ES5 Harmony derived proxy traps. */
    virtual bool has(JSContext *cx, JSObject *proxy, jsid id, bool *bp)
    {
        return mTraps.has ?
               mTraps.has(cx, proxy, id, bp) :
               BaseProxyHandler::has(cx, proxy, id, bp);
    }

    virtual bool hasOwn(JSContext *cx, JSObject *proxy, jsid id, bool *bp)
    {
        return mTraps.hasOwn ?
               mTraps.hasOwn(cx, proxy, id, bp) :
               BaseProxyHandler::hasOwn(cx, proxy, id, bp);
    }

    virtual bool get(JSContext *cx, JSObject *proxy, JSObject *receiver,
                     jsid id, JS::Value *vp)
    {
        return mTraps.get ?
                mTraps.get(cx, proxy, receiver, id, vp) :
                BaseProxyHandler::get(cx, proxy, receiver, id, vp);
    }

    virtual bool set(JSContext *cx, JSObject *proxy, JSObject *receiver,
                     jsid id, bool strict, JS::Value *vp)
    {
        return mTraps.set ?
                mTraps.set(cx, proxy, receiver, id, strict, vp) :
                BaseProxyHandler::set(cx, proxy, receiver, id, strict, vp);
    }

    virtual bool keys(JSContext *cx, JSObject *proxy, JS::AutoIdVector &props)
    {
        return mTraps.keys ?
                mTraps.keys(cx, proxy, props) :
                BaseProxyHandler::keys(cx, proxy, props);
    }

    virtual bool iterate(JSContext *cx, JSObject *proxy, unsigned flags,
                         JS::Value *vp)
    {
        return mTraps.iterate ?
                mTraps.iterate(cx, proxy, flags, vp) :
                BaseProxyHandler::iterate(cx, proxy, flags, vp);
    }

    /* Spidermonkey extensions. */
    virtual bool call(JSContext *cx, JSObject *proxy, unsigned argc, JS::Value *vp)
    {
        return mTraps.call ?
                mTraps.call(cx, proxy, argc, vp) :
                BaseProxyHandler::call(cx, proxy, argc, vp);
    }

    virtual bool construct(JSContext *cx, JSObject *proxy, unsigned argc, JS::Value *argv, JS::Value *rval)
    {
        return mTraps.construct ?
                mTraps.construct(cx, proxy, argc, argv, rval) :
                BaseProxyHandler::construct(cx, proxy, argc, argv, rval);
    }

    virtual bool nativeCall(JSContext *cx, JS::IsAcceptableThis test, JS::NativeImpl impl, JS::CallArgs args)
    {
        return mTraps.nativeCall ?
                mTraps.nativeCall(cx, test, impl, args) :
                BaseProxyHandler::nativeCall(cx, test, impl, args);
    }

    virtual bool hasInstance(JSContext *cx, JSObject *proxy, const JS::Value *vp, bool *bp)
    {
        return mTraps.hasInstance ?
                mTraps.hasInstance(cx, proxy, vp, bp) :
                BaseProxyHandler::hasInstance(cx, proxy, vp, bp);
    }

    virtual JSType typeOf(JSContext *cx, JSObject *proxy)
    {
        return mTraps.typeOf ?
                mTraps.typeOf(cx, proxy) :
                BaseProxyHandler::typeOf(cx, proxy);
    }

    virtual bool objectClassIs(JSObject *obj, js::ESClassValue classValue, JSContext *cx)
    {
        return mTraps.objectClassIs ?
                mTraps.objectClassIs(obj, classValue, cx) :
                BaseProxyHandler::objectClassIs(obj, classValue, cx);
    }

    virtual JSString *obj_toString(JSContext *cx, JSObject *proxy)
    {
        return mTraps.obj_toString ?
                mTraps.obj_toString(cx, proxy) :
                BaseProxyHandler::obj_toString(cx, proxy);
    }

    virtual JSString *fun_toString(JSContext *cx, JSObject *proxy, unsigned indent)
    {
        return mTraps.fun_toString ?
                mTraps.fun_toString(cx, proxy, indent) :
                BaseProxyHandler::fun_toString(cx, proxy, indent);
    }

    /*virtual bool regexp_toShared(JSContext *cx, JSObject *proxy, RegExpGuard *g)
    {
        return mTraps.regexp_toShared ?
                mTraps.regexp_toShared(cx, proxy, g) :
                BaseProxyHandler::regexp_toShared(cx, proxy, g);
                }*/

    virtual bool defaultValue(JSContext *cx, JSObject *obj, JSType hint, JS::Value *vp)
    {
        return mTraps.defaultValue ?
                mTraps.defaultValue(cx, obj, hint, vp) :
                BaseProxyHandler::defaultValue(cx, obj, hint, vp);
    }

    virtual bool iteratorNext(JSContext *cx, JSObject *proxy, JS::Value *vp)
    {
        return mTraps.iteratorNext ?
                mTraps.iteratorNext(cx, proxy, vp) :
                BaseProxyHandler::iteratorNext(cx, proxy, vp);
    }

    virtual void finalize(JSFreeOp *fop, JSObject *proxy)
    {
        return mTraps.finalize ?
                mTraps.finalize(fop, proxy) :
                BaseProxyHandler::finalize(fop, proxy);
    }

    virtual bool getElementIfPresent(JSContext *cx, JSObject *obj, JSObject *receiver,
                                     uint32_t index, JS::Value *vp, bool *present)
    {
        return mTraps.getElementIfPresent ?
                mTraps.getElementIfPresent(cx, obj, receiver, index, vp, present) :
                BaseProxyHandler::getElementIfPresent(cx, obj, receiver, index, vp, present);
    }

    virtual bool getPrototypeOf(JSContext *cx, JSObject *proxy, JSObject **proto)
    {
        return mTraps.getPrototypeOf ?
                mTraps.getPrototypeOf(cx, proxy, proto) :
                BaseProxyHandler::getPrototypeOf(cx, proxy, proto);
    }
};

typedef union {
    uint64_t u64v;
    jsval jsv;
} jsval_u64_t;

static inline uint64_t 
jsval_to_uint64(jsval v)
{
    jsval_u64_t conv;
    conv.jsv = v;
    return conv.u64v;
}

static inline jsval
uint64_to_jsval(uint64_t v)
{
    jsval_u64_t conv;
    conv.u64v = v;
    return conv.jsv;
}

extern "C" {

void*
GetJSClassHookStubPointer(enum StubType type)
{
    switch (type) {
    case PROPERTY_STUB:
        return (void*)JS_PropertyStub;
    case STRICT_PROPERTY_STUB:
        return (void*)JS_StrictPropertyStub;
    case ENUMERATE_STUB:
        return (void*)JS_EnumerateStub;
    case CONVERT_STUB:
        return (void*)JS_ConvertStub;
    case RESOLVE_STUB:
        return (void*)JS_ResolveStub;
    }
    return NULL;
}

JSBool
RUST_JSVAL_IS_NULL(uint64_t v)
{
    return JSVAL_IS_NULL(uint64_to_jsval(v));
}

JSBool
RUST_JSVAL_IS_VOID(uint64_t v)
{
    return JSVAL_IS_VOID(uint64_to_jsval(v));
}

JSBool
RUST_JSVAL_IS_INT(uint64_t v)
{
    return JSVAL_IS_INT(uint64_to_jsval(v));
}

int32_t
RUST_JSVAL_TO_INT(uint64_t v)
{
    return JSVAL_TO_INT(uint64_to_jsval(v));
}

uint64_t
RUST_INT_TO_JSVAL(int32_t v)
{
    return jsval_to_uint64(INT_TO_JSVAL(v));
}

JSBool
RUST_JSVAL_IS_DOUBLE(uint64_t v)
{
    return JSVAL_IS_DOUBLE(uint64_to_jsval(v));
}

double
RUST_JSVAL_TO_DOUBLE(uint64_t v)
{
    return JSVAL_TO_DOUBLE(uint64_to_jsval(v));
}

uint64_t
RUST_DOUBLE_TO_JSVAL(double v)
{
    return jsval_to_uint64(DOUBLE_TO_JSVAL(v));
}

uint64_t
RUST_UINT_TO_JSVAL(uint32_t v)
{
    return jsval_to_uint64(UINT_TO_JSVAL(v));
}

JSBool
RUST_JSVAL_IS_NUMBER(uint64_t v)
{
    return JSVAL_IS_NUMBER(uint64_to_jsval(v));
}

JSBool
RUST_JSVAL_IS_STRING(uint64_t v)
{
    return JSVAL_IS_STRING(uint64_to_jsval(v));
}

JSString *
RUST_JSVAL_TO_STRING(uint64_t v)
{
    return JSVAL_TO_STRING(uint64_to_jsval(v));
}

uint64_t
RUST_STRING_TO_JSVAL(JSString *v)
{
    return jsval_to_uint64(STRING_TO_JSVAL(v));
}

JSBool
RUST_JSVAL_IS_OBJECT(uint64_t v)
{
    jsval jsv = uint64_to_jsval(v);
    return !JSVAL_IS_PRIMITIVE(jsv) || JSVAL_IS_NULL(jsv);
}

JSObject *
RUST_JSVAL_TO_OBJECT(uint64_t v)
{
    return JSVAL_TO_OBJECT(uint64_to_jsval(v));
}

uint64_t
RUST_OBJECT_TO_JSVAL(JSObject *v)
{
    return jsval_to_uint64(OBJECT_TO_JSVAL(v));
}

JSBool
RUST_JSVAL_IS_BOOLEAN(uint64_t v)
{
    return JSVAL_IS_BOOLEAN(uint64_to_jsval(v));
}

JSBool
RUST_JSVAL_TO_BOOLEAN(uint64_t v)
{
    return JSVAL_TO_BOOLEAN(uint64_to_jsval(v));
}

uint64_t
RUST_BOOLEAN_TO_JSVAL(JSBool v)
{
    return jsval_to_uint64(BOOLEAN_TO_JSVAL(v));
}

JSBool
RUST_JSVAL_IS_PRIMITIVE(uint64_t v)
{
    return JSVAL_IS_PRIMITIVE(uint64_to_jsval(v));
}

JSBool
RUST_JSVAL_IS_GCTHING(uint64_t v)
{
    return JSVAL_IS_GCTHING(uint64_to_jsval(v));
}

void *
RUST_JSVAL_TO_GCTHING(uint64_t v)
{
    return JSVAL_TO_GCTHING(uint64_to_jsval(v));
}

uint64_t
RUST_PRIVATE_TO_JSVAL(void *v)
{
    return jsval_to_uint64(PRIVATE_TO_JSVAL(v));
}

void *
RUST_JSVAL_TO_PRIVATE(uint64_t v)
{
    return JSVAL_TO_PRIVATE(uint64_to_jsval(v));
}

uint64_t
RUST_JS_NumberValue(double d)
{
    return jsval_to_uint64(JS_NumberValue(d));
}

const JSJitInfo*
RUST_FUNCTION_VALUE_TO_JITINFO(jsval* v)
{
    return FUNCTION_VALUE_TO_JITINFO(*v);
}

JSBool
CallJitPropertyOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, jsval *vp)
{
    struct {
        JSObject** obj;
    } tmp = { &thisObj };
    return ((JSJitPropertyOp)info->op)(cx, *reinterpret_cast<JSHandleObject*>(&tmp), specializedThis, vp);
}

JSBool
CallJitMethodOp(JSJitInfo *info, JSContext* cx, JSObject* thisObj, void *specializedThis, uint argc, jsval *vp)
{
    struct {
        JSObject** obj;
    } tmp = { &thisObj };
    return ((JSJitMethodOp)info->op)(cx, *reinterpret_cast<JSHandleObject*>(&tmp), specializedThis, argc, vp);
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
CreateProxyHandler(const ProxyTraps* aTraps)
{
    return new ForwardingProxyHandler(*aTraps);
}

JSObject*
NewProxyObject(JSContext* aCx, void* aHandler, const js::Value* priv,
               JSObject* proto, JSObject* parent, JSObject* call,
               JSObject* construct)
{
    return js::NewProxyObject(aCx, (js::BaseProxyHandler*)aHandler, *priv, proto,
                              parent, call, construct);
}

uint64_t
GetProxyExtra(JSObject* obj, uint slot)
{
    return jsval_to_uint64(js::GetProxyExtra(obj, slot));
}

uint64_t
GetProxyPrivate(JSObject* obj)
{
    return jsval_to_uint64(js::GetProxyPrivate(obj));
}

JSObject*
GetObjectProto(JSObject* obj)
{
    return js::GetObjectProto(obj);
}

JSBool
RUST_JSID_IS_INT(jsid id)
{
    return JSID_IS_INT(id);
}

int
RUST_JSID_TO_INT(jsid id)
{
    return JSID_TO_INT(id);
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

JSClass*
GetObjectJSClass(JSObject* obj)
{
    return js::GetObjectJSClass(obj);
}

JSErrorFormatString*
js_GetErrorMessage(void* userRef, char* locale, uint errorNumber)
{
    return js_GetErrorMessage(userRef, locale, errorNumber);
}

} // extern "C"
