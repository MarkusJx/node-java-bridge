#ifndef NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP
#define NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP

#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    class java_instance_proxy : public Napi::ObjectWrap<java_instance_proxy> {
    public:
        static Napi::Function getConstructor(Napi::Env env, const Napi::Object &class_proxy);

        static Napi::Value fromJObject(Napi::Env env, const jni::jobject_wrapper<jobject> &obj,
                                        const Napi::Object &class_proxy);

        explicit java_instance_proxy(const Napi::CallbackInfo &info);

        jni::jobject_wrapper<jobject> object;
        std::shared_ptr<jni::java_class> clazz;
        jni::jvm_wrapper jvm;
        std::string classname;

    private:
        static Napi::Value staticGetter(const Napi::CallbackInfo &info);
        static void staticSetter(const Napi::CallbackInfo &info, const Napi::Value& value);

        static Napi::Value callStaticFunction(const Napi::CallbackInfo &info);
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP
