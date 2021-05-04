#ifndef NODE_JAVA_BRIDGE_JVM_CONTAINER_HPP
#define NODE_JAVA_BRIDGE_JVM_CONTAINER_HPP

#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    /**
     * This class statically manages the root jvm_wrapper
     * which contains the root jvm_jvm, which manages
     * the jvm pointers and destroys the jvm on its destruction.
     */
    class jvm_container {
    public:
        /**
         * Create the static jvm_container instance.
         * This will create the root jvm_jvm, which will
         * load the shared library and create the actual
         * Java VM instance.
         *
         * @param lib_path the path to the jvm shared library
         * @param version the jvm version to use
         */
        static void createInstance(const std::string &lib_path, jint version);

        /**
         * Destroy the Java vm instance.
         * This will immediately cause all
         * subsequent calls to the vm to fail.
         */
        static void destroyInstance();

        /**
         * Create the jvm_container.
         * Don't use this, use createInstance instead.
         * This is only public as the unique_ptr uses it.
         *
         * @param lib_path the path to the jvm shared library
         * @param version the jvm version to use
         */
        explicit jvm_container(const std::string &lib_path, jint version);

        /**
         * Get a reference to the root jvm_wrapper.
         * In most cases, it will be wiser to use attachJvm
         * instead to attach the current thread to the jvm.
         * Will throw an std::runtime_error if the static
         * instance was never initialized (or is already deleted).
         *
         * @return the reference to the root jvm_wrapper
         */
        static jni::jvm_wrapper &getJvm();

        /**
         * Attach the current thread to the jvm.
         * Internally uses getJvm, therefore, this will
         * also throw an std::runtime_error if the static
         * instance was never initialized.
         * Returns a jni_wrapper with the current environment
         * if the current thread is the same as the thread in
         * which the jvm was originally created. This operation
         * is a no-op, but keep in mind that we will create some
         * overhead with this operation as the shared jvm object
         * (pointer) needs to be copied in order for this to work.
         *
         * @return the jni_wrapper with the attached environment
         */
        static jni::jni_wrapper attachJvm();

        /**
         * Destroy the jvm_container instance.
         * This will release the jvm instance into
         * the wild green yonder and cause all
         * subsequent calls to the jvm to fail.
         */
        ~jvm_container();

    private:
        // The root jvm_wrapper instance
        jni::jvm_wrapper root_jvm;

        // The static jvm_container instance
        static std::unique_ptr<jvm_container> instance;
    };
}

#endif //NODE_JAVA_BRIDGE_JVM_CONTAINER_HPP
