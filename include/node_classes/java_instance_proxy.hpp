#ifndef NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP
#define NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP

#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    /**
     * A java class instance proxy
     */
    class java_instance_proxy : public Napi::ObjectWrap<java_instance_proxy> {
    public:
        /**
         * Get the constructor function
         *
         * @param env the environment to work in
         * @param class_proxy the class proxy to use
         * @return the constructor function
         */
        static Napi::Function getConstructor(Napi::Env env, const Napi::Object &class_proxy);

        /**
         * Create a java instance proxy from an existing jobject
         *
         * @param env the environment to work in
         * @param obj the object to copy
         * @param class_proxy the class proxy to use
         * @return the created java instance proxy
         */
        static Napi::Value fromJObject(Napi::Env env, const jni::jobject_wrapper<jobject> &obj,
                                       const Napi::Object &class_proxy);

        /**
         * Create a new java instance proxy instance.
         * This will either create a new java class instance
         * or load in an existing one if passed as the first argument.
         * Arguments:
         *      info[0] {node_jobject_wrapper} the jobject to create this from
         *      or
         *      info {any[]} the constructor arguments
         *
         * @param info the callback info
         */
        explicit java_instance_proxy(const Napi::CallbackInfo &info);

        void Finalize(Napi::Env env) override;

        /**
         * The java instance proxy destructor
         */
        ~java_instance_proxy() override;

        // The stored class instance object
        jni::jobject_wrapper<jobject> object;
        // The class
        std::shared_ptr<jni::java_class> clazz;
        // The class name
        std::string classname;

    private:
        /**
         * A getter for static member fields
         *
         * @param info the callback info
         * @return the converted value
         */
        static Napi::Value staticGetter(const Napi::CallbackInfo &info);

        /**
         * A setter for static member fields
         *
         * @param info the callback info
         * @param value the value to set
         */
        static void staticSetter(const Napi::CallbackInfo &info, const Napi::Value &value);

        /**
         * Call a static function
         *
         * @param info the callback info. Must contain the call arguments
         * @return the value returned by the function
         */
        static Napi::Value callStaticFunction(const Napi::CallbackInfo &info);

        /**
         * Call a static function.
         * Async version,
         *
         * @param info the callback info. Must contain the call arguments
         * @return the promise which will will return the function call return value
         */
        static Napi::Value callStaticFunctionAsync(const Napi::CallbackInfo &info);

        /**
         * Generate the properties for the class
         *
         * @param class_proxy the class proxy to use
         * @param env the environment to work in
         * @return the class properties
         */
        static std::vector<Napi::ObjectWrap<java_instance_proxy>::PropertyDescriptor>
        generateProperties(const Napi::Object &class_proxy, const Napi::Env &env);

        /**
         * Create a new class instance
         *
         * @param info the callback info
         * @return the created class
         */
        static Napi::Value newInstance(const Napi::CallbackInfo &info);

        /**
         * Get the java class type
         */
        static Napi::Value get_class(const Napi::CallbackInfo &info);

        /**
         * Check if this is instance of another class
         * Arguments:
         *      info[0] {string} the name of the class to check if this is an instance of
         *
         * @param info the callback info
         * @return true if this is instance of the other class
         */
        Napi::Value instanceOf(const Napi::CallbackInfo &info);
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_INSTANCE_PROXY_HPP
