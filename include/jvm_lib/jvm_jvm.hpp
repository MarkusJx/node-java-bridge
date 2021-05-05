#ifndef NODE_JAVA_BRIDGE_JVM_JVM_HPP
#define NODE_JAVA_BRIDGE_JVM_JVM_HPP

#include <mutex>

namespace jni {
    /**
     * A wrapper for the java vm
     */
    class jvm_jvm {
    public:
        /**
         * Create a jvm wrapper with an existing jvm pointer
         *
         * @param vm the jvm pointer to store
         */
        explicit jvm_jvm(JavaVM *vm);

        /**
         * Get the environment.
         * The same as JavaVM->GetEnv().
         *
         * @param env pointer to the location where the JNI interface pointer
         *            for the current thread will be placed.
         * @param version The requested JNI version.
         * @return If the current thread is not attached to the VM, sets *env to NULL,
         *         and returns JNI_EDETACHED. If the specified version is not supported, sets
         *         *env to NULL, and returns JNI_EVERSION. Otherwise, sets *env to the
         *         appropriate interface, and returns JNI_OK.
         */
        jint GetEnv(void **env, jint version);

        /**
         * Attaches the current thread to a Java VM.
         * The same as JavaVM->AttachCurrentThread().
         *
         * @param env pointer to the location where the JNI interface
         *            pointer of the current thread will be placed
         * @param options can be NULL or a pointer to a JavaVMAttachArgs
         *                structure to specify additional information
         * @return JNI_OK on success; returns a suitable JNI error
         *         code (a negative number) on failure
         */
        jint AttachCurrentThread(void **env, void *options);

        /**
         * Detaches the current thread from a Java VM.
         * All Java monitors held by this thread are released.
         * All Java threads waiting for this thread to die are notified.
         *
         * @return JNI_OK on success; returns a suitable JNI error code
         *         (a negative number) on failure
         */
        jint DetachCurrentThread();

        /**
         * Force reset the Java vm.
         * This will call DestroyJavaVM on the jvm instance
         * to destroy the vm. Any vm calls after this will
         * throw an std::runtime_error as the jvm is now invalid.
         * Also, valid() will now return false.
         *
         * This should not be called unless it's really required.
         */
        void forceReset();

        /**
         * Check if the java vm is valid.
         * Will return false if the jvm_jvm
         * object was initialized with a null
         * jvm pointer or forceReset() was called.
         *
         * @return true if the vm is valid
         */
        bool valid();

        /**
         * Destroys the jvm if it is valid
         */
        ~jvm_jvm();

    private:
        // A mutex used to synchronize all vm
        // operations. Just exists so no vm operation
        // is executed as the vm is destroyed.
        std::mutex mtx;

        // The pointer to the java vm
        JavaVM *jvm;
    };
}

#endif //NODE_JAVA_BRIDGE_JVM_JVM_HPP
