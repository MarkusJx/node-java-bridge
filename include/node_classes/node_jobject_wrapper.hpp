#ifndef NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP

#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    class node_jobject_wrapper : public Napi::ObjectWrap<node_jobject_wrapper> {
    public:
        static void init(Napi::Env env, Napi::Object &exports);

        static Napi::Object createInstance();

        static bool instanceOf(const Napi::Object &obj);

        explicit node_jobject_wrapper(const Napi::CallbackInfo &info);

        void setData(const jni::jobject_wrapper<jobject> &obj);

        [[nodiscard]] jni::jobject_wrapper<jobject> getObject() const;

    private:
        static Napi::FunctionReference *constructor;

        jni::jobject_wrapper<jobject> object;
    };
}

#endif //NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP
