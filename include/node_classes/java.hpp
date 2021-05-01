#ifndef NODE_JAVA_BRIDGE_JAVA_HPP
#define NODE_JAVA_BRIDGE_JAVA_HPP

#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    class java : public Napi::ObjectWrap<java> {
    public:
        static void init(Napi::Env env, Napi::Object &exports);

        explicit java(const Napi::CallbackInfo &info);

        Napi::Value getClass(const Napi::CallbackInfo &info);

        Napi::Object getClass(const Napi::Env &env, const std::string &classname);

        void appendToClasspath(const Napi::CallbackInfo &info);

        ~java() override;

        jni::jvm_wrapper java_environment;

    private:
        Napi::Value getLoadedJars(const Napi::CallbackInfo &info);

        std::vector<std::string> loaded_jars;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_HPP
