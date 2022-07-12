#ifndef NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP

#include <functional>
#include <vector>
#include <map>
#include <jni.h>
#include <util/shared_releaser.hpp>
#include <definitions.hpp>

#include "shared_library.hpp"
#include "jobject_wrapper.hpp"
#include "java_type.hpp"

/**
 * A namespace for jni operations
 */
namespace jni {
    /**
     * A namespace for jni function types
     */
    namespace jni_types {
        /**
         * The JNI_CreateJavaVM function type
         */
        using JNI_CreateJavaVM_t = decltype(::JNI_CreateJavaVM);
    }

    /**
     * A java class constructor
     */
    class java_constructor;

    /**
     * A java class field
     */
    class java_field;

    /**
     * A java class member function
     */
    class java_function;

    /**
     * A java class
     */
    class java_class;

    /**
     * The main jni wrapper
     */
    class jni_wrapper {
    public:
        /**
         * Create an empty jni wrapper.
         * Can't do nothing.
         */
        jni_wrapper() noexcept;

        /**
         * Create a jni wrapper from a jvm environment
         *
         * @param env the environment to use. Should be valid.
         */
        explicit jni_wrapper(jvm_env &&env);

        jni_wrapper(const jni_wrapper &other) = delete;

        /**
         * Check for a jvm error and
         * throw it if an error exists
         */
        void checkForError() const;

        /**
         * Convert a string to a java string
         *
         * @param str the string to convert
         * @return the converted and wrapped java string
         */
        JAVA_NODISCARD jobject_wrapper<jstring> string_to_jstring(const std::string &str) const;

        /**
         * Convert a java string to a C++ string
         *
         * @param str the string to convert
         * @param convertErrors whether to convert any thrown errors
         *                      and throw them as java errors or throw C++ errors
         * @return the converted string
         */
        JAVA_NODISCARD std::string jstring_to_string(jstring str, bool convertErrors = true) const;

        /**
         * Get all constructors for a class
         *
         * @param className the name of the class to get the constructors from
         * @return the list of constructors for that class
         */
        JAVA_NODISCARD std::vector<java_constructor> getClassConstructors(const std::string &className) const;

        /**
         * Get java.lang.Class
         *
         * @return the java.lang.Class jclass
         */
        JAVA_NODISCARD jclass getJavaLangClass() const;

        /**
         * Get a class by its name
         *
         * @param className the name of the class to resolve
         * @return the java.lang.Class object. Can be casted to jclass.
         */
        JAVA_NODISCARD jobject_wrapper<jobject> getClassByName(const std::string &className) const;

        /**
         * Get the class's fields
         *
         * @param className the name of the class to get the fields from
         * @param onlyStatic whether to return only static or only non-static fields
         * @return the list of class member fields
         */
        JAVA_NODISCARD std::vector<java_field> getClassFields(const std::string &className, bool onlyStatic) const;

        /**
         * Get a class's declared methods
         *
         * @param className the name of the class to get the methods from
         * @param onlyStatic whether to return only static or only non-static methods
         * @return the list of class member functions
         */
        JAVA_NODISCARD std::vector<java_function>
        getClassFunctions(const std::string &className, bool onlyStatic) const;

        /**
         * Get a java class instance.
         * This will create a java_class instance
         * and populate it with all fields,
         * methods and constructors to be used.
         *
         * @param className the name of the class to resolve
         * @return the created java_class instance
         */
        JAVA_NODISCARD java_class getClass(const std::string &className) const;

        /**
         * Get the jclass to a class name.
         * Please note that the class is created using
         * java methods and there is only a limited
         * amount of local refs, so this should be converted
         * to a global ref if it should exist for a longer time.
         *
         * @param className the name of the class to resolve
         * @return the resolved jclass
         */
        JAVA_NODISCARD jclass getJClass(const std::string &className) const;

        /**
         * Find a class by its name.
         * Does not use the dot notation.
         *
         * @param className the class name, e.g. 'java/lang/String'
         * @param convert_exceptions whether to convert exceptions
         * @return the found jclass
         */
        JAVA_NODISCARD jclass find_class(const std::string &className, bool convert_exceptions = true) const;

        /**
         * Throw the last exception
         *
         * @param line the line number this was called from
         */
        void throwLastException(int line = -1) const;

        /**
         * Append a jar file to the class path.
         * This will not alter the actual class path,
         * it will rather create a class loader instance
         * with the jar loaded and sets the old class
         * loader as its parent.
         *
         * @param path the jar to append to the class path
         */
        void appendClasspath(const std::string &path);

        /**
         * Append multiple jars to the class path.
         * Does the same as the other appendClasspath
         * method but with multiple jar paths.
         *
         * @param paths the paths to append to the class path
         */
        void appendClasspath(const std::vector<std::string> &paths);

        /**
         * Check if a class can be assigned to another one.
         * Mostly used to determine if a class extends
         * another one and can therefore be passed as an
         * argument even though only the superclass is
         * requested as a parameter.
         *
         * @param sub the child class to check
         * @param sup the parent class to check
         * @return true if sub is assignable to sup
         */
        JAVA_NODISCARD bool class_is_assignable(const std::string &sub, const std::string &sup) const;

        /**
         * Get the class name of a jobject.
         * This we can use to convert a java.lang.Object to its original type.
         *
         * @param obj the object to get the class's name from
         * @return the objects class name
         */
        JAVA_NODISCARD std::string getObjectClassName(jobject obj) const;

        /**
         * Convert a jobject to a jint-
         * The object's class must be java.lang.Integer.
         *
         * @param obj the object to convert
         * @return the converted jint
         */
        JAVA_NODISCARD jint jobject_to_jint(jobject obj) const;

        /**
         * Convert a jobject to a jboolean.
         * The object's class must be java.lang.Boolean.
         *
         * @param obj the object to convert
         * @return the converted jboolean
         */
        JAVA_NODISCARD jboolean jobject_to_jboolean(jobject obj) const;

        /**
         * Convert a jobject to a jbyte.
         * The object's class must be java.lang.Byte.
         *
         * @param obj the object to convert
         * @return the converted jbyte
         */
        JAVA_NODISCARD jbyte jobject_to_jbyte(jobject obj) const;

        /**
         * Convert a jobject to a jchar.
         * THe object's class must be java.lang.Character.
         *
         * @param obj the object to convert
         * @return the converted jchar
         */
        JAVA_NODISCARD jchar jobject_to_jchar(jobject obj) const;

        /**
         * Convert a jobject to a jshort.
         * The object's type must be java.lang.Short.
         *
         * @param obj the object to convert
         * @return the converted jshort
         */
        JAVA_NODISCARD jshort jobject_to_jshort(jobject obj) const;

        /**
         * Convert a jobject to a jlong.
         * The object's type must be java.lang.Long.
         *
         * @param obj the object to convert
         * @return the converted jlong
         */
        JAVA_NODISCARD jlong jobject_to_jlong(jobject obj) const;

        /**
         * Convert a jobject to a jfloat.
         * The object's type must be java.lang.Float.
         *
         * @param obj the object to convert
         * @return the converted jfloat
         */
        JAVA_NODISCARD jfloat jobject_to_jfloat(jobject obj) const;

        /**
         * Convert a jobject to a jdouble.
         * The object's type must be java.lang.Double.
         *
         * @param obj the object to convert
         * @return the converted jdouble
         */
        JAVA_NODISCARD jdouble jobject_to_jdouble(jobject obj) const;

        /**
         * Convert a jint to a jobject
         *
         * @param e the jint to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jint(jint e) const;

        /**
         * Convert a jshort to a jobject
         *
         * @param e the jshort to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jshort(jshort e) const;

        /**
         * Convert a jdouble to a jobject
         *
         * @param e the jdouble to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jdouble(jdouble e) const;

        /**
         * Convert a jfloat to a jobject
         *
         * @param e the jfloat to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jfloat(jfloat e) const;

        /**
         * Convert a jlong to a jobject
         *
         * @param e the jlong to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jlong(jlong e) const;

        /**
         * Convert a jint to a jobject
         *
         * @param e the jint to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jbyte(jbyte e) const;

        /**
         * Convert a jchar to a jobject
         *
         * @param e the jchar to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jchar(jchar e) const;

        /**
         * Convert a jboolean to a jobject
         *
         * @param e the jboolean to convert
         * @return the converted jobject
         */
        JAVA_NODISCARD jobject_wrapper<jobject> create_jboolean(jboolean e) const;

        /**
         * Attach a java environment to the current
         * java vm and return the created jni_wrapper.
         * If the current thread is the same as the
         * thread the vm was created in, a copy of
         * this jni_wrapper instance will be returned.
         *
         * @return the created jni_wrapper in the new thread
         */
        JAVA_NODISCARD jni_wrapper attachEnv() const;

        /**
         * Operator-> for conveniently calling jni functions
         *
         * @return the jni env pointer
         */
        JAVA_NODISCARD JNIEnv *operator->() const;

        /**
         * Get the class loader instance
         *
         * @return the class loader instance
         */
        static const jobject_wrapper<jobject> &getClassloader();

        /**
         * Check if this jni_wrapper was initialized
         *
         * @return true if it was initialized and is ready for use
         */
        operator bool() const;

        // The jvm environment to use
        const jvm_env env;

    protected:
        /**
         * Get the system's default class loader
         *
         * @return the default class loader
         */
        jobject_wrapper<jobject> getSystemClassLoader();

        // Whether this was initialized with an environment
        bool initialized;
        // The static class loader instance
        // This will be replaced once a new jar is loaded
        static jobject_wrapper<jobject> classLoader;
    };

    /**
     * The jvm wrapper.
     * Basically a version of the jni_wrapper
     * which is capable of creating a jvm.
     * This will also hold the root jvm_env
     * instance and may only exist once.
     * Equal in functionality otherwise.
     */
    class jvm_wrapper final : public jni_wrapper {
    public:
        /**
         * Create an empty jvm wrapper instance.
         * Can't do anything.
         */
        jvm_wrapper() noexcept;

        /**
         * Create a new jvm wrapper instance.
         * This will create a new jvm instance.
         * Please note that there can only be one
         * simultaneous jvm instance in one program
         * instance. The function call will throw
         * if a jvm is already running.
         *
         * @param jvmPath the path to the jvm shared library
         * @param version the jvm version to use
         */
        static jvm_wrapper create_jvm_wrapper(const std::string &jvmPath, jint version);

        /**
         * The JNI_CreateJavaVM version loaded dynamically from the jvm
         */
        std::function<jni_types::JNI_CreateJavaVM_t> JNI_CreateJavaVM = nullptr;
    private:
        jvm_wrapper(jvm_env &&env, std::function<jni_types::JNI_CreateJavaVM_t> &&createVm);

        // The shared library instance
        static shared_library library;
    };

    /**
     * A java class field
     */
    class java_field {
    public:
        /**
         * Create a java field instance
         *
         * @param signature the fields signature
         * @param name the fields name
         * @param id the fields id
         * @param isStatic whether the field is static
         * @param isFinal whether the field is final
         */
        java_field(const std::string &signature, std::string name, jfieldID id, bool isStatic, bool isFinal);

        /**
         * Get the field's value.
         * The field must be non-static for this to work.
         *
         * @param classInstance the class instance to get the value from
         * @param data the jobject_wrapper to store a potential jobject in
         * @return the field's value
         */
        jvalue get(jobject classInstance, jobject_wrapper<jobject> &data) const;

        /**
         * Get the field's value.
         * The field must be static for this to work.
         *
         * @param clazz the class to get the field value from
         * @param data the jobject_wrapper to store a potential jobject in
         * @return the field's value
         */
        jvalue getStatic(jclass clazz, jobject_wrapper<jobject> &data) const;

        /**
         * Set the field's value.
         * The field must be non-static for this to work.
         *
         * @param classInstance the class instance
         * @param data the data to set
         */
        void set(jobject classInstance, jvalue data) const;

        /**
         * Set the field's value.
         * The field must be static for this to work.
         *
         * @param clazz the class to set the field
         * @param data the data to set
         */
        void setStatic(jclass clazz, jvalue data) const;

        // The field's signature
        java_type signature;
        // The field's name
        std::string name;
        // Whether the field is static
        bool isStatic;
        // Whether the field is final
        bool isFinal;
        // The field's id
        jfieldID id;
    };

    /**
     * A java class member function
     */
    class java_function {
    public:
        /**
         * Create a java_function instance
         *
         * @param parameterTypes the parameter types
         * @param returnType the return type
         * @param functionName the function's name
         * @param method the method id
         * @param isStatic whether the function is static
         */
        java_function(std::vector<java_type> parameterTypes, java_type returnType, std::string functionName,
                      jmethodID method, bool isStatic);

        /**
         * Convert the function to a string
         *
         * @return the function as a human-readable string
         */
        JAVA_NODISCARD std::string to_string() const;

        // The function's parameter types
        std::vector<java_type> parameterTypes;
        // The function's return type signature
        java_type returnType;
        // The function's name
        std::string name;
        // Whether the function is static
        bool isStatic;
        // The method id
        jmethodID method;
    };

    /**
     * A java class constructor
     */
    class java_constructor : private jobject_wrapper<jobject> {
    public:
        /**
         * Create a java_constructor instance
         *
         * @param object the constructor object
         * @param jni the jni_wrapper to use
         */
        java_constructor(jobject object, const jni_wrapper &jni);

        /**
         * Create a new class instance
         *
         * @param args the instance arguments
         * @return the created class instance
         */
        JAVA_NODISCARD jobject_wrapper<jobject> newInstance(const std::vector<jobject_wrapper<jobject>> &args) const;

        /**
         * Convert this constructor to a string
         *
         * @return the constructor as a human-readable string
         */
        JAVA_NODISCARD std::string to_string() const;

        // The constructor's parameter types
        std::vector<java_type> parameterTypes;

    private:
        /**
         * Get the constructor's parameter types
         *
         * @return the parameter types
         */
        JAVA_NODISCARD std::vector<java_type> getParameterTypes() const;
    };

    /**
     * A java class
     */
    class java_class {
    public:
        /**
         * Create an empty java_class instance
         */
        java_class();

        /**
         * Create a java_class instance
         *
         * @param static_fields the static class fields
         * @param fields the class fields
         * @param static_functions the static member functions
         * @param functions the member functions
         * @param constructors the class's constructors
         * @param clazz the class
         */
        java_class(const std::vector<java_field> &static_fields, const std::vector<java_field> &fields,
                   const std::vector<java_function> &static_functions, const std::vector<java_function> &functions,
                   std::vector<java_constructor> constructors, jobject_wrapper<jclass> clazz);

        // The static and non-static fields.
        // Contains the field name as a key and the field as a value
        std::map<std::string, java_field> static_fields, fields;
        // The static and non-static member functions
        // Contains the method name as a key and the method as a value
        std::map<std::string, std::vector<java_function>> static_functions, functions;
        // The class's constructors
        std::vector<java_constructor> constructors;
        // The class
        jobject_wrapper<jclass> clazz;
    };

} // namespace jni

#endif //NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
