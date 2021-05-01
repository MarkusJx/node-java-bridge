#ifndef NODE_JAVA_BRIDGE_JVM_ENV_HPP
#define NODE_JAVA_BRIDGE_JVM_ENV_HPP

#include <jni.h>
#include <shared_releaser.hpp>

namespace jni {
    /**
     * A java environment container
     */
    class jvm_env : public shared_releaser {
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
        jvm_env(JavaVM *vm, JNIEnv *env, jint version, bool detachThread = false);

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

        // The java virtual machine pointer.
        // This is equal on all jvm_env instances.
        JavaVM *jvm;

        // The JNI environment pointer.
        // This value may differ on different
        // jvm_env instances as different instances
        // may be attached to different threads.
        JNIEnv *env;

        // The java version used to create the vm
        jint version;
    };
}

#endif //NODE_JAVA_BRIDGE_JVM_ENV_HPP
