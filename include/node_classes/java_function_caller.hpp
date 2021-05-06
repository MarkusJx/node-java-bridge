#ifndef NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP
#define NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP

#include <napi.h>
#include <jni.h>
#include <napi_tools.hpp>

#include "jvm_lib/jobject_wrapper.hpp"

namespace node_classes {
    class java_function_caller : public Napi::ObjectWrap<java_function_caller> {
    public:
        class value_converter;

        using callback = napi_tools::callbacks::callback<value_converter(jobjectArray)>;

        static void setLibraryPath(const std::string &path, const std::string &workingDir);

        static void init(Napi::Env env, Napi::Object &exports);

        static bool instanceOf(const Napi::Object &object);

        explicit java_function_caller(const Napi::CallbackInfo &info);

        ~java_function_caller() override;

        std::map<std::string, callback> functions;
        jni::jobject_wrapper<jobject> proxy;
    private:
        jclass clazz;
        jni::jobject_wrapper<jobject> object;

        static Napi::FunctionReference *constructor;
        static std::string nativeLibPath;
        static std::vector<char> classData;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_FUNCTION_CALLER_HPP
