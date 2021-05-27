#ifndef NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP
#define NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP

#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>

#include "jvm_lib/jobject_wrapper.hpp"

namespace node_classes {
    /**
     * A class for calling node functions from java,
     * using java.lang.reflect.Proxy extending
     * java interfaces and overriding the functions.
     */
    class java_function_caller : public Napi::ObjectWrap<java_function_caller> {
    public:
        /**
         * A class for converting the returned
         * n-api values to java objects
         */
        class value_converter;

        /**
         * The callback function type
         */
        using callback = napi_tools::callbacks::callback<value_converter(jobjectArray, JNIEnv *)>;

        /**
         * Set the native library (node_java_bridge.node) path and the
         * working directory. Will load the JavaFunctionCaller.class file.
         *
         * @param path the path to the native library for java to load
         * @param workingDir the current working directory
         */
        static void setLibraryPath(const std::string &path, const std::vector<char> &classData);

        /**
         * Add the class to the exports
         *
         * @param env the environment to work in
         * @param exports the module exports
         */
        static void init(Napi::Env &env, Napi::Object &exports);

        /**
         * Check if an object is an instance of java_function_caller
         *
         * @param object the object to check against
         * @return true if the object is instance of java_function_caller
         */
        static bool instanceOf(const Napi::Object &object);

        /**
         * Create a new java_function_caller instance.
         * Will create a new io.github.markusjx.bridge.JavaFunctionCaller
         * class instance and initiate a java.lang.reflect.Proxy with it.
         * Arguments:
         *      info[0] {string} the name of the interface to instantiate
         *      info[1] {object} the object containing the function to override
         *
         * @param info the callback info
         */
        explicit java_function_caller(const Napi::CallbackInfo &info);

        /**
         * Get the name of the instantiated interface
         *
         * @return the interface's signature
         */
        JAVA_NODISCARD const std::string &getClassName() const;

        /**
         * Destroy the java_function_caller instance.
         * Will call destruct() on the JavaFunctionCaller
         * java class instance to render that class instance invalid
         * and useless, it will throw an exception if someone
         * tries to call a member function from this point on.
         */
        ~java_function_caller() override;

        /**
         * The js call back functions with the
         * function names as their key
         */
        std::map<std::string, callback> functions;

        /**
         * The java.lang.reflect.Proxy instance
         */
        jni::jobject_wrapper<jobject> proxy;
    private:
        // The io.github.markusjx.bridge.JavaFunctionCaller class
        jclass clazz;
        // The io.github.markusjx.bridge.JavaFunctionCaller
        // class instance
        jni::jobject_wrapper<jobject> object;
        // The class name
        std::string classname;

        // The java_function_caller constructor
        static Napi::FunctionReference *constructor;

        // The path to the native library (node_java_bridge.node)
        static std::string nativeLibPath;
        // The raw bytes from the JavaFunctionCaller.class file
        static std::vector<char> classData;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP
