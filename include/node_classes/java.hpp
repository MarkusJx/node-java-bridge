#ifndef NODE_JAVA_BRIDGE_JAVA_HPP
#define NODE_JAVA_BRIDGE_JAVA_HPP

#include <napi.h>

#include "jvm_lib/jni_wrapper.hpp"
#include "util/persistent_object.hpp"

namespace node_classes {
    /**
     * The java object.
     * This actually can't really do anything,
     * as it is just here to provide some convenient
     * methods to be used by node.js. The jvm instance
     * is nowadays managed by another class but this still
     * creates this instance to be used by all other methods.
     *
     * This is also responsible for deleting the jvm instance
     * as deleting it at any other point may cause the jvm
     * to crash. You can find further information about that
     * in the destructor of node_classes::jvm_container.
     * Because of this, the jvm only lives as long as the java
     * class is referenced in node.js, which is until the module
     * is unloaded or a new java instance is created.
     *
     * Properties:
     *      version {string} the jvm version in use
     *      wantedVersion {string} the version defined
     *                    on construct or the default jvm version
     */
    class java : public Napi::ObjectWrap<java> {
    public:
        /**
         * Add the class to the module exports
         *
         * @param env the environment to work in
         * @param exports the exports to write to
         */
        static void init(Napi::Env env, Napi::Object &exports);

        static void set_root_dir(const std::string &root_dir);

        /**
         * Set the native library (node_java_bridge.node) path and the
         * working directory. Will load the JavaFunctionCaller.class file.
         *
         * @param path the path to the native library for java to load
         */
        static void set_native_lib_path(const std::string &path);

        /**
         * Create a new java class instance.
         * This will create the jvm in node_classes::jvm_container.
         * Please not that only one jvm instance may exist at a time.
         * If null is passed as a version argument, one of the two versions
         * will be selected: 1.6, 1.8; The selection is based on whether
         * the module was compiled with the headers of a jvm with a
         * version >= 1.8.
         * Arguments:
         *      info[0] {string} the path to the jvm shared library
         *      info[1] {string | null} an optional version argument
         * @param info the callback info
         */
        explicit java(const Napi::CallbackInfo &info);

        /**
         * Get a class (this should be called from javascript)
         * Arguments:
         *      info[0] {string} the name of the class to resolve
         *
         * @param info the callback info
         * @return the created class proxy
         */
        static Napi::Value getClass(const Napi::CallbackInfo &info);

        /**
         * Create a new class proxy instance.
         * Async version, may only be called from javascript.
         * Arguments:
         *      info[0] {string} the name of the class to resolve
         *
         * @param info the callback info
         * @return the newly created promise
         */
        static Napi::Value getClassAsync(const Napi::CallbackInfo &info);

        /**
         * Get a class (this can only be called from inside this module)
         *
         * @param env the environment to work in
         * @param classname the name of the class to resolve
         * @return the class proxy instance
         */
        static Napi::Object getClass(const Napi::Env &env, const std::string &classname);

        /**
         * Append a jar to the class path.
         * This will not actually alter the class path,
         * take a look at jni::jni_wrapper::appendClasspath
         * for further information.
         * Arguments:
         *      info[0] {string | array<string>} the path(s) to
         *              the file(s) to append to the classpath
         *
         * @param info the callback info
         */
        void appendToClasspath(const Napi::CallbackInfo &info);

        /**
         * Append to the classpath. Async version.
         * This has basically the same implementation
         * and syntax as node_classes::java::appendToClasspath,
         * so take a look at that implementation for further information.
         *
         * @param info the callback info
         * @return the promise created by this call
         */
        Napi::Value appendToClasspathAsync(const Napi::CallbackInfo &info);

        /**
         * If set to true, attach new threads as daemon threads
         */
        static const std::atomic_bool &use_daemon_threads();

        static void set_config(const Napi::CallbackInfo &info, const Napi::Value &value);

        static Napi::Value get_config(const Napi::CallbackInfo &info);

        /**
         * Destroy the jvm instance.
         * Will release the vm and make all calls to
         * the jvm throw exceptions since the vm is dead now.
         * Arguments: None
         *
         * @param info the callback info
         */
        static void destroyJVM(const Napi::CallbackInfo &info);

        /**
         * Destroy the java object.
         * This will also unload the jvm.
         * See node_classes::jvm_container::~jvm_container
         * for further information.
         */
        ~java() override;

    private:
        /**
         * Get the loaded jars in the class path
         * Arguments: none
         *
         * @param info the callback info
         * @return the loaded jars as Napi::Strings in a Napi::Array
         */
        Napi::Value getLoadedJars(const Napi::CallbackInfo &info);

        // The list of loaded jar files
        std::vector<std::string> loaded_jars;
        static std::string root_dir;
        // The path to the native library (node_java_bridge.node)
        static std::string nativeLibPath;
        static std::atomic_bool _use_daemon_threads;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_HPP
