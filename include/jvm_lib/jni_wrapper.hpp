#ifndef NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP

#include <functional>
#include <vector>
#include <map>
#include <jni.h>
#include <shared_releaser.hpp>

#include "shared_library.hpp"
#include "jobject_wrapper.hpp"

namespace jni {
    namespace jni_types {
        using JNI_CreateJavaVM_t = decltype(::JNI_CreateJavaVM);
    }

    class java_constructor;

    class java_field;

    class java_function;

    class java_class;

    class jni_wrapper {
    public:
        jni_wrapper() noexcept;

        explicit jni_wrapper(jvm_env env);

        void checkForError() const;

        [[nodiscard]] jobject_wrapper<jstring> string_to_jstring(const std::string &str) const;

        [[nodiscard]] std::string jstring_to_string(jstring str, bool convertErrors = true) const;

        [[nodiscard]] std::vector<java_constructor> getClassConstructors(const std::string &className) const;

        [[nodiscard]] jclass getJavaLangClass() const;

        [[nodiscard]] jobject_wrapper<jobject> getClassByName(const std::string &className) const;

        [[nodiscard]] std::vector<java_field> getClassFields(const std::string &className, bool onlyStatic) const;

        [[nodiscard]] std::vector<java_function> getClassFunctions(const std::string &className, bool onlyStatic) const;

        [[nodiscard]] java_class getClass(const std::string &className) const;

        [[nodiscard]] jclass getJClass(const std::string &className) const;

        void throwLastException(int line = -1) const;

        void appendClasspath(const std::string& path);

        [[nodiscard]] bool class_is_assignable(const std::string &sub, const std::string &sup) const;

        [[nodiscard]] std::string get_object_class_name(jobject obj) const;

        [[nodiscard]] jint jobject_to_jint(jobject obj) const;

        [[nodiscard]] jboolean jobject_to_jboolean(jobject obj) const;

        [[nodiscard]] jbyte jobject_to_jbyte(jobject obj) const;

        [[nodiscard]] jchar jobject_to_jchar(jobject obj) const;

        [[nodiscard]] jshort jobject_to_jshort(jobject obj) const;

        [[nodiscard]] jlong jobject_to_jlong(jobject obj) const;

        [[nodiscard]] jfloat jobject_to_jfloat(jobject obj) const;

        [[nodiscard]] jdouble jobject_to_jdouble(jobject obj) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jint(jint e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jshort(jshort e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jdouble(jdouble e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jfloat(jfloat e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jlong(jlong e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jbyte(jbyte e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jchar(jchar e) const;

        [[nodiscard]] jobject_wrapper<jobject> create_jboolean(jboolean e) const;

        [[nodiscard]] JNIEnv *operator->() const;

        operator jvm_env() const;

        operator bool() const;

    protected:
        jobject_wrapper<jobject> getSystemClassLoader();

        jvm_env env;
        bool initialized;
        static jobject_wrapper<jobject> classLoader;
    };

    class jvm_wrapper final : public jni_wrapper {
    public:
        jvm_wrapper() noexcept;

        jvm_wrapper(const std::string &jvmPath, jint version);

        [[nodiscard]] jni_wrapper attachEnv() const;

        std::function<jni_types::JNI_CreateJavaVM_t> JNI_CreateJavaVM = nullptr;
    private:
        shared_library library;
        jint version;
    };

    class java_field {
    public:
        java_field(std::string signature, std::string name, jfieldID id, bool isStatic, bool isFinal, jni_wrapper env);

        jobject_wrapper<jobject> get(jobject classInstance) const;

        jobject_wrapper<jobject> getStatic(jclass clazz) const;

        void set(jobject classInstance, jobject data) const;

        void setStatic(jclass clazz, jobject data) const;

        std::string signature;
        std::string name;
        bool isStatic;
        bool isFinal;
        jfieldID id;

    private:
        jni_wrapper env;
    };

    class java_function {
    public:
        java_function(std::vector<std::string> parameterTypes, std::string returnType, std::string functionName,
                      jmethodID method, bool isStatic, jni_wrapper env);

        [[nodiscard]] std::string to_string() const;

        std::vector<std::string> parameterTypes;
        std::string returnType;
        std::string name;
        bool isStatic;
        jmethodID method;

    private:
        const jni_wrapper env;
    };

    class java_constructor : public jobject_wrapper<jobject> {
    public:
        java_constructor(jobject object, const jni_wrapper &jni);

        [[nodiscard]] jobject_wrapper<jobject> newInstance(const std::vector<jobject_wrapper<jobject>> &args) const;

        [[nodiscard]] std::string to_string() const;

        std::vector<std::string> parameterTypes;

    private:
        [[nodiscard]] std::vector<std::string> getParameterTypes() const;

        jni_wrapper jni;
    };

    class java_class {
    public:
        java_class(const std::vector<java_field> &static_fields, const std::vector<java_field> &fields,
                   const std::vector<java_function> &static_functions, const std::vector<java_function> &functions,
                   std::vector<java_constructor> constructors, jclass clazz);

        std::map<std::string, java_field> static_fields, fields;
        std::map<std::string, std::vector<java_function>> static_functions, functions;
        std::vector<java_constructor> constructors;
        jclass clazz;
    };

} // namespace jni

#endif //NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
