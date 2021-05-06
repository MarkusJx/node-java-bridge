#ifndef NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
#define NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP

#include <napi.h>

#include "jvm_lib/jni_wrapper.hpp"

/**
 * A namespace for type conversions between
 * java values and n-api values.
 * Can also handle function calls.
 */
namespace conversion_helper {
    /**
     * Convert a jobject to a n-api value
     *
     * @param env the environment to work in
     * @param obj the jobject to convert
     * @param signature the signature of the object to convert
     * @return the converted n-api value
     */
    Napi::Value jobject_to_value(const Napi::Env &env, const jni::jobject_wrapper<jobject> &obj,
                                 const std::string &signature, bool objects = true);

    /**
     * Convert a napi value to a jobject
     *
     * @param env the environment to use
     * @param value the value to convert
     * @param signature the jobjects signature
     * @param objects whether to convert values to java.lang.Object, if supplied
     * @return the converted jobject
     */
    jni::jobject_wrapper<jobject> value_to_jobject(const Napi::Env &env, const Napi::Value &value,
                                                   const std::string &signature, bool objects);

    /**
     * Convert a napi_valuetype to a string
     *
     * @param type the valuetype to get as a string
     * @return the type as a string
     */
    std::string napi_valuetype_to_string(napi_valuetype type);

    /**
     * Find a matching constructor to n-api arguments
     *
     * @param args the args to find a matching constructor to
     * @param constructors the list of constructors
     * @param outArgs the n-api args converted to jobjects
     * @param error the error string. Will be set if the call fails
     * @return the constructor pointer or nullptr if no constructor was found
     */
    const jni::java_constructor *find_matching_constructor(const Napi::CallbackInfo &args,
                                                           const std::vector<jni::java_constructor> &constructors,
                                                           std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                           std::string &error);

    /**
     * Call a matching function
     *
     * @param args the arguments to find a matching function to
     * @param classInstance the class instance to call the function on
     * @param functions the list of functions
     * @return the value returned by the function call
     */
    Napi::Value call_matching_function(const Napi::CallbackInfo &args,
                                       const jni::jobject_wrapper<jobject> &classInstance,
                                       const std::vector<jni::java_function> &functions);

    /**
     * Call a matching static function
     *
     * @param info the arguments to find a matching function to
     * @param clazz the class to call the function on
     * @param functions the list of possible matches
     * @return the value returned by the function call
     */
    Napi::Value call_matching_static_function(const Napi::CallbackInfo &info, jclass clazz,
                                              const std::vector<jni::java_function> &functions);


    /**
     * Find a matching function
     *
     * @param args the arguments to find a matching function to
     * @param functions the list of possible function matches
     * @param outArgs the converted java argument list to write to
     * @param error the error message. Will be set if the call fails.
     * @param outValues the function arguments as jvalues. Will be bound to outArgs.
     * @return the function pointer or nullptr if no matching function was found
     */
    const jni::java_function *find_matching_function(const Napi::CallbackInfo &args,
                                                     const std::vector<jni::java_function> &functions,
                                                     std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                     std::string &error, std::vector<jvalue> &outValues);

    /**
     * Call a function
     *
     * @param function the function to call
     * @param classInstance the class instance to call the function on
     * @param args the function call arguments
     * @return the value returned by the function
     */
    jvalue call_function(const jni::java_function &function, const jni::jobject_wrapper<jobject> &classInstance,
                         const std::vector<jvalue> &args);

    /**
     * Call a static function
     *
     * @param function the function to call
     * @param clazz the class to call the function on
     * @param args the function call arguments
     * @return the value returned by the function
     */
    jvalue call_static_function(const jni::java_function &function, jclass clazz, const std::vector<jvalue> &args);

    /**
     * Convert a jvalue to a Napi::Value
     *
     * @param value the value to convert
     * @param signature the signature of the java value
     * @param env the environment to work in
     * @return the converted Napi::Value
     */
    Napi::Value jvalue_to_napi_value(jvalue value, const std::string &signature, const Napi::Env &env);

    /**
     * Convert a n-api value to a jvalue
     *
     * @param env the environment to work in
     * @param value the value to convert
     * @param signature the java signature of the value
     * @param values the values. This is just here to keep jobjects alive
     * @param objects whether to convert values to java.lang.Object, if supplied
     * @return the converted value
     */
    jvalue napi_value_to_jvalue(const Napi::Env &env, const Napi::Value &value, const std::string &signature,
                                std::vector<jni::jobject_wrapper<jobject>> &values, bool objects);

    /**
     * Convert a Napi::Array to a jarray
     *
     * @param env the environment to work in
     * @param j_env the java environment to work in
     * @param signature the signature of the java array
     * @param array the array to convert
     * @param objects whether to convert values to java.lang.Object, if supplied
     * @return the converted array
     */
    jni::jobject_wrapper<jarray> napi_array_to_jarray(const Napi::Env &env, const jni::jni_wrapper &j_env,
                                                      const std::string &signature, const Napi::Array &array,
                                                      bool objects);

    /**
     * Check if a number is an integer value (not floating point).
     * See: https://github.com/nodejs/node-addon-api/issues/265
     *
     * @param env the environment to work in
     * @param num the number to check
     * @return true if the value is not of type floating point
     */
    bool is_integer(const Napi::Env &env, const Napi::Number &num);
}

#endif //NODE_JAVA_BRIDGE_CONVERSION_HELPER_HPP
