#ifndef NODE_JAVA_BRIDGE_JAVA_CLASS_PROXY_HPP
#define NODE_JAVA_BRIDGE_JAVA_CLASS_PROXY_HPP

#include <mutex>
#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    /**
     * A java class proxy class<br>
     *<br>
     * JS Properties:<br>
     * class.name {string} The java class name<br>
     * java.instance {object} The node_classes::java instance as a javascript object
     */
    class java_class_proxy : public Napi::ObjectWrap<java_class_proxy> {
    public:
        static void init(Napi::Env env, Napi::Object &exports);

        static Napi::Object createInstance(const Napi::Object &java_instance, const Napi::String &classname);

        /**
         * Create a java class proxy.
         * This takes a java class (see java.hpp) instance as a first argument
         * and a string representing the class to fetch as a second argument.
         *
         * @param info the callback info containing the arguments
         */
        explicit java_class_proxy(const Napi::CallbackInfo &info);

        Napi::Value getClassConstructor(const Napi::CallbackInfo &info);

        std::shared_ptr<jni::java_class> clazz;

        std::mutex mtx;

        jni::jvm_wrapper jvm;

        std::string classname;

        static Napi::FunctionReference *constructor;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_CLASS_PROXY_HPP
