#ifndef NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP

#include <napi.h>
#include "jvm_lib/jni_wrapper.hpp"

namespace node_classes {
    /**
     * A n-api wrapper around a jobject
     */
    class node_jobject_wrapper : public Napi::ObjectWrap<node_jobject_wrapper> {
    public:
        /**
         * Initialize the node_jobject_wrapper
         *
         * @param env the environment to work in
         * @param exports the exports to write to
         */
        static void init(Napi::Env env, Napi::Object &exports);

        /**
         * Create a new node_jobject_wrapper instance
         *
         * @return the created instance
         */
        static Napi::Object createInstance();

        /**
         * Check if an object is a node_jobject_wrapper
         *
         * @param obj the object to check
         * @return true if the object is a node_jobject_wrapper
         */
        static bool instanceOf(const Napi::Object &obj);

        /**
         * Create a node_jobject_wrapper.
         * Arguments: none
         *
         * @param info the callback info
         */
        explicit node_jobject_wrapper(const Napi::CallbackInfo &info);

        /**
         * Set the data
         *
         * @param obj the jobject to store
         */
        void setData(const jni::jobject_wrapper<jobject> &obj);

        /**
         * Get the stored jobject
         *
         * @return the stored jobject
         */
        JAVA_NODISCARD jni::jobject_wrapper<jobject> getObject() const;

    private:
        // The class's constructor
        static Napi::FunctionReference *constructor;

        // The stored jobject
        jni::jobject_wrapper<jobject> object;
    };
}

#endif //NODE_JAVA_BRIDGE_NODE_JOBJECT_WRAPPER_HPP
