#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/java.hpp"
#include "util.hpp"

#include <iostream>

Napi::Value
conversion_helper::static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env,
                                               const Napi::Object &java_instance) {
    return jobject_to_value(env, java_instance, to_convert.getStatic(clazz), to_convert.signature);
}

Napi::Value conversion_helper::jobject_to_value(const Napi::Env &env, const Napi::Object &java_instance,
                                                const jni::jobject_wrapper<jobject> &obj,
                                                const std::string &signature) {
    node_classes::java *java_ptr = Napi::ObjectWrap<node_classes::java>::Unwrap(java_instance);
    if (!java_ptr) {
        throw Napi::Error::New(env);
    }

    jni::jni_wrapper j_env = java_ptr->java_environment.attachEnv();
    if (signature == "I") {
        // Value is an integer
        return Napi::Number::New(env, j_env.jobject_to_jint(obj));
    } else if (signature == "Z") {
        // Value is a boolean
        return Napi::Boolean::New(env, j_env.jobject_to_jboolean(obj));
    } else if (signature == "B") {
        // Value is a byte
        return Napi::Number::New(env, j_env.jobject_to_jbyte(obj));
    } else if (signature == "C") {
        // Value is a char
        return Napi::String::New(env, std::string(1, (char) j_env.jobject_to_jchar(obj)));
    } else if (signature == "S") {
        // Value is a short
        return Napi::Number::New(env, j_env.jobject_to_jshort(obj));
    } else if (signature == "J") {
        // Value is a long
        // TODO: Return raw values
        return Napi::Number::New(env, (double) j_env.jobject_to_jlong(obj));
    } else if (signature == "F") {
        // Value is a float
        return Napi::Number::New(env, j_env.jobject_to_jfloat(obj));
    } else if (signature == "D") {
        // Value is a double
        return Napi::Number::New(env, j_env.jobject_to_jdouble(obj));
    } else if (signature == "Ljava/lang/String;" || signature == "java.lang.String") {
        // Value is a string
        return Napi::String::New(env,
                                 j_env.jstring_to_string(reinterpret_cast<jstring>(obj.obj)));
    } else if (signature[0] == '[') {
        // The value is an array
        jni::jobject_wrapper<jobjectArray> j_array = obj.as<jobjectArray>();
        const jsize size = j_env->GetArrayLength(j_array);
        Napi::Array array = Napi::Array::New(env, static_cast<size_t>(size));

        for (jsize i = 0; i < size; i++) {
            auto cur = jni::jobject_wrapper(j_env->GetObjectArrayElement(j_array, i), j_env);
            array.Set(i, jobject_to_value(env, java_instance, cur, signature.substr(1)));
        }

        return array;
    } else {
        // The value is a class instance
        const std::string classname = util::string_replace(signature.substr(1, signature.length() - 2), '/', '.');
        Napi::Object class_proxy = java_ptr->getClass(env, classname);

        return node_classes::java_instance_proxy::fromJObject(env, obj, class_proxy);
    }
}

#define CHECK_TYPE_MATCH(check, type) \
if (!value.check())                   \
    throw Napi::TypeError::New(env, "Expected type " #type " but got " + napi_valuetype_to_string(value.Type()))

jni::jobject_wrapper<jobject> conversion_helper::value_to_jobject(const Napi::Env &env, const jni::jni_wrapper &j_env,
                                                                  const Napi::Value &value,
                                                                  const std::string &signature) {
    if (value.IsNull()) {
        return jni::jobject_wrapper<jobject>();
    }

    if (signature == "I") {
        // Value is an integer
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jint(value.ToNumber().operator int());
    } else if (signature == "Z") {
        // Value is a boolean
        CHECK_TYPE_MATCH(IsBoolean, boolean);
        return j_env.create_jboolean(value.ToBoolean());
    } else if (signature == "B") {
        // Value is a byte
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jbyte((jbyte) value.ToNumber().operator int());
    } else if (signature == "C") {
        // Value is a char
        CHECK_TYPE_MATCH(IsString, string);
        return j_env.create_jchar(value.ToString().Utf8Value()[0]);
    } else if (signature == "S") {
        // Value is a short
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jshort((jshort) value.ToNumber().operator int());
    } else if (signature == "J") {
        // Value is a long
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jlong((jlong) value.ToNumber().operator long long());
    } else if (signature == "F") {
        // Value is a float
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jfloat(value.ToNumber().operator float());
    } else if (signature == "D") {
        // Value is a double
        CHECK_TYPE_MATCH(IsNumber, number);
        return j_env.create_jdouble(value.ToNumber().operator double());
    } else if (signature == "Ljava/lang/String;" || signature == "java.lang.String") {
        // Value is a string
        CHECK_TYPE_MATCH(IsString, string);
        return j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>();
    } else if (signature[0] == '[') {
        CHECK_TYPE_MATCH(IsArray, array);
        auto array = value.As<Napi::Array>();
        std::string classname = signature.substr(1);
        jclass clazz = j_env->FindClass(classname.c_str());
        j_env.checkForError();

        jint array_size = static_cast<jint>(array.Length());

        jni::jobject_wrapper<jobjectArray> j_array(j_env->NewObjectArray(array_size, clazz, nullptr), j_env);
        j_env.checkForError();

        for (jint i = 0; i < array_size; i++) {
            j_env->SetObjectArrayElement(j_array, i, value_to_jobject(env, j_env, array.Get(i), classname));
            j_env.checkForError();
        }

        return j_array.as<jobject>();
    } else {
        // Expecting a class instance
        CHECK_TYPE_MATCH(IsObject, object);
        Napi::Object obj = value.ToObject();
        std::string expected_classname = signature.substr(1, signature.size() - 2);

        if (obj.InstanceOf(
                Napi::ObjectWrap<node_classes::java_instance_proxy>::DefineClass(env, "java_instance_proxy", {}))) {
            Napi::Object class_proxy = obj.Get("class_proxy_instance").ToObject();
            std::string classname = util::string_replace(class_proxy.Get("class_name").ToString().Utf8Value(), '.',
                                                         '/');

            if (classname != expected_classname) {
                throw std::runtime_error("Expected class " + expected_classname + " but got " + classname);
            }

            return Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(obj)->object;
        } else {
            throw Napi::TypeError::New(env, "Expected instance proxy, but got something else");
        }
    }
}

std::string conversion_helper::napi_valuetype_to_string(napi_valuetype type) {
    switch (type) {
        case napi_undefined:
            return "undefined";
        case napi_null:
            return "null";
        case napi_boolean:
            return "boolean";
        case napi_number:
            return "number";
        case napi_string:
            return "string";
        case napi_symbol:
            return "symbol";
        case napi_object:
            return "object";
        case napi_function:
            return "function";
        case napi_external:
            return "external";
        case napi_bigint:
            return "bigint";
        default:
            return "unknown";
    }
}

bool value_type_matches_signature(const Napi::Value &value, const std::string &signature) {
    if (value.IsNull()) {
        return true;
    } else if (value.IsBoolean()) {
        return signature == "Z";
    } else if (value.IsNumber()) {
        return signature == "B" || signature == "S" || signature == "I" || signature == "J" || signature == "F" ||
               signature == "D";
    } else if (value.IsString()) {
        return signature == "java.lang.String" || (value.ToString().Utf8Value().size() == 1 && signature == "C") ||
               signature == "Ljava/lang/String;";
    } else if (value.IsArray()) {
        if (!signature.empty() && signature[0] == '[') {
            const auto array = value.As<Napi::Array>();
            if (array.IsEmpty()) {
                return true;
            } else {
                const uint32_t zero = 0;
                return value_type_matches_signature(array.Get(zero), signature.substr(1));
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

bool args_match_java_types(const Napi::CallbackInfo &args, const std::vector<std::string> &parameterTypes) {
    if (args.Length() != parameterTypes.size()) {
        return false;
    }

    for (size_t i = 0; i < parameterTypes.size(); i++) {
        if (!value_type_matches_signature(args[i], parameterTypes[i])) {
            return false;
        }
    }

    return true;
}

std::vector<jni::jobject_wrapper<jobject>> args_to_java_arguments(const Napi::CallbackInfo &args,
                                                                  const jni::jni_wrapper &j_env,
                                                                  const std::vector<std::string> &parameterTypes) {
    std::vector<jni::jobject_wrapper<jobject>> arguments;
    for (size_t i = 0; i < args.Length(); i++) {
        arguments.push_back(conversion_helper::value_to_jobject(args.Env(), j_env, args[i], parameterTypes[i]));
    }

    return arguments;
}

jni::jobject_wrapper<jobject> conversion_helper::match_constructor_arguments(const Napi::CallbackInfo &args,
                                                                             const jni::jni_wrapper &j_env,
                                                                             const std::vector<jni::java_constructor> &constructors) {
    for (const auto &c : constructors) {
        if (args_match_java_types(args, c.parameterTypes)) {
            return c.newInstance(args_to_java_arguments(args, j_env, c.parameterTypes));
        }
    }

    throw Napi::TypeError::New(args.Env(), "Could not find an appropriate constructor");
}

void jobject_to_jvalue(const std::string &signature, const jni::jobject_wrapper<jobject> &arg,
                       const jni::jni_wrapper &env, jvalue &val) {
    if (signature == "I") {
        // Value is an integer
        val.i = env.jobject_to_jint(arg);
    } else if (signature == "Z") {
        // Value is a boolean
        val.z = env.jobject_to_jboolean(arg);
    } else if (signature == "B") {
        // Value is a byte
        val.b = env.jobject_to_jbyte(arg);
    } else if (signature == "C") {
        // Value is a char
        val.c = env.jobject_to_jchar(arg);
    } else if (signature == "S") {
        // Value is a short
        val.s = env.jobject_to_jshort(arg);
    } else if (signature == "J") {
        // Value is a long
        val.j = env.jobject_to_jlong(arg);
    } else if (signature == "F") {
        // Value is a float
        val.f = env.jobject_to_jfloat(arg);
    } else if (signature == "D") {
        // Value is a double
        val.d = env.jobject_to_jdouble(arg);
    } else {
        // Value is some kind of object
        val.l = arg.obj;
    }
}

std::vector<jvalue> jobjects_to_jvalues(const std::vector<std::string> &parameterTypes, const jni::jni_wrapper &j_env,
                                        const std::vector<jni::jobject_wrapper<jobject>> &args) {
    std::vector<jvalue> values(parameterTypes.size());
    for (size_t i = 0; i < parameterTypes.size(); i++) {
        jobject_to_jvalue(parameterTypes[i], args[i], j_env, values[i]);
    }

    return values;
}

#define CALL_FUNCTION(functionName) auto res = j_env->functionName(classInstance, function.method, args.data()); \
                                    j_env.checkForError()

Napi::Value call_function(const jni::java_function &function, const jni::jobject_wrapper<jobject> &classInstance,
                          const jni::jni_wrapper &j_env, const Napi::Env &env, const std::vector<jvalue> &args,
                          const Napi::Object &javaInstance) {
    const std::string &signature = function.returnType;
    if (signature == "V") {
        // Method returns void
        j_env->CallVoidMethodA(classInstance, function.method, args.data());
        j_env.checkForError();
        return env.Undefined();
    } else if (signature == "I") {
        // Value is an integer
        CALL_FUNCTION(CallIntMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "Z") {
        // Value is a boolean
        CALL_FUNCTION(CallBooleanMethodA);
        return Napi::Boolean::New(env, res);
    } else if (signature == "B") {
        // Value is a byte
        CALL_FUNCTION(CallByteMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "C") {
        // Value is a char
        CALL_FUNCTION(CallCharMethodA);
        return Napi::String::New(env, std::string({static_cast<char>(res)}));
    } else if (signature == "S") {
        // Value is a short
        CALL_FUNCTION(CallShortMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "J") {
        // Value is a long
        CALL_FUNCTION(CallLongMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "F") {
        // Value is a float
        CALL_FUNCTION(CallFloatMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "D") {
        // Value is a double
        CALL_FUNCTION(CallDoubleMethodA);
        return Napi::Number::From(env, res);
    } else {
        // Value is some kind of object
        CALL_FUNCTION(CallObjectMethodA);
        jni::jobject_wrapper<jobject> obj(res, j_env);
        return conversion_helper::jobject_to_value(env, javaInstance, obj, signature);
    }
}

Napi::Value conversion_helper::call_matching_function(const Napi::CallbackInfo &args, const Napi::Object &java_instance,
                                                      const jni::jobject_wrapper<jobject> &classInstance,
                                                      const std::vector<jni::java_function> &functions) {
    jni::jni_wrapper j_env = Napi::ObjectWrap<node_classes::java>::Unwrap(java_instance)->java_environment.attachEnv();

    for (const auto &f : functions) {
        if (args_match_java_types(args, f.parameterTypes)) {
            std::vector<jni::jobject_wrapper<jobject>> j_args = args_to_java_arguments(args, j_env, f.parameterTypes);
            std::vector<jvalue> values = jobjects_to_jvalues(f.parameterTypes, j_env, j_args);

            return call_function(f, classInstance, j_env, args.Env(), values, java_instance);
        }
    }

    throw Napi::TypeError::New(args.Env(), "Could not find a matching function");
}
