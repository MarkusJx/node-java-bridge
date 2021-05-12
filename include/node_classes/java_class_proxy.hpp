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
     */
    class java_class_proxy : public Napi::ObjectWrap<java_class_proxy> {
    public:
        /**
         * Write the class proxy class to the module's exports
         *
         * @param env the environment to use
         * @param exports the exports to write to
         */
        static void init(Napi::Env env, Napi::Object &exports);

        /**
         * Create a class_proxy instance
         *
         * @param classname the name of the class to resolve
         * @return the created instance
         */
        static Napi::Object createInstance(const Napi::String &classname);

        /**
         * Create a new java_class_proxy instance.
         * Async version.
         *
         * @param classname the name of the class to resolve
         * @param env the environment to work in
         * @return the newly created promise
         */
        static Napi::Value createInstanceAsync(const std::string &classname, const Napi::Env &env);

        /**
         * Create a java class proxy.
         * This takes a string representing the class to fetch as its first argument.
         *
         * @param info the callback info containing the arguments
         */
        explicit java_class_proxy(const Napi::CallbackInfo &info);

        /**
         * Get the class's constructor.
         * Arguments: none
         *
         * @param info the callbackInfo
         * @return
         */
        Napi::Value getClassConstructor(const Napi::CallbackInfo &info);

        /**
         * The java_class_proxy destructor
         */
        ~java_class_proxy() override;

        /**
         * This will check if this was the last instance
         * referencing the stored class and will delete
         * the class instance from the cache if so.
         * This will also call reset on the pointer passed to it.
         *
         * @param ptr the shared pointer to release
         * @param classname the name of the stored class
         */
        static void cleanup_class(std::shared_ptr<jni::java_class> &ptr, const std::string &classname);

        // The jni::java_class instance
        std::shared_ptr<jni::java_class> clazz;

        // A mutex for synchronization
        std::mutex mtx;

        // The name of the class
        std::string classname;

        // A vector to store additional data to be freed on destruction
        // This is used to store names of the sync calls to any function
        // (functionName + "Sync") to keep this information as long as the
        // class is in use.
        std::vector<std::unique_ptr<char, decltype(&free)>> additionalData;

        // The constructor pointer
        static Napi::FunctionReference *constructor;

    private:
        static std::mutex cache_mtx;
        static std::map<std::string, std::shared_ptr<jni::java_class>> cached_classes;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_CLASS_PROXY_HPP
