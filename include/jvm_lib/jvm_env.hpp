#ifndef NODE_JAVA_BRIDGE_JVM_ENV_HPP
#define NODE_JAVA_BRIDGE_JVM_ENV_HPP

#include <jni.h>
#include <util/shared_releaser.hpp>

#include "jvm_jvm.hpp"

namespace jni {
    /**
     * A java environment container
     */
    class jvm_env {
    public:
        /**
         * Create a null java environment
         */
        jvm_env() noexcept;

        /**
         * Create a java environment
         *
         * @param vm the java virtual machine instance
         * @param env the jni environment
         * @param version the java version to use
         * @param detachThread whether to detach the thread on destruction rather than destroying the whole vm
         */
        jvm_env(const std::shared_ptr<jvm_jvm> &vm, JNIEnv *env, jint version, bool detachThread = false);

        /**
         * Attach the current thread to the jvm
         *
         * @return the attached env
         */
        [[nodiscard]] jvm_env attach_env() const;

        /**
         * Operator->
         * Does what it says on the box.
         *
         * @return the jni environment pointer
         */
        JNIEnv *operator->() const;

        /**
         * Check if the jvm (and environment) is still valid.
         * Just checks if the JavaVM pointer and the
         * JNIEnv pointers are not nullptr
         *
         * @return true if the environment is valid
         */
        [[nodiscard]] bool valid() const;

        /**
         * Force reset the java environment.
         * This should not be called unless required,
         * as the jvm will be destroyed if it isn't
         * referenced anymore.
         *
         * This calls jvm_jvm.forceReset() to reset
         * the jvm, to see the implementation details,
         * take a look at that implementation in
         * <jvm_lib/jvm_jvm.hpp>.
         */
        void forceReset();

        // The java virtual machine pointer.
        // This is equal on all jvm_env instances.
        std::shared_ptr<jvm_jvm> jvm;

        // The java version used to create the vm
        jint version;

    private:
        // The JNI environment pointer.
        // This value may differ on different
        // jvm_env instances as different instances
        // may be attached to different threads.
        JNIEnv *env;

        // The releaser responsible for detaching
        // the current environment from the jvm
        // (if required)
        shared_releaser envReleaser;
    };
}

#endif //NODE_JAVA_BRIDGE_JVM_ENV_HPP
