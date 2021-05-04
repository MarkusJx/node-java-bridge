#include <iostream>

#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/jvm_container.hpp"
#include "node_classes/java.hpp"
#include "util/util.hpp"

#define TRY_RUN(statement) try { \
                                statement;\
                            } catch (const std::exception &e) {\
                                throw Napi::Error::New(env, e.what()); \
                            }

Napi::Value
conversion_helper::static_java_field_to_object(const jni::java_field &to_convert, jclass clazz, const Napi::Env &env) {
    return jobject_to_value(env, to_convert.getStatic(clazz), to_convert.signature);
}

Napi::Value conversion_helper::jobject_to_value(const Napi::Env &env, const jni::jobject_wrapper<jobject> &obj,
                                                std::string signature) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    signature = util::make_java_name_readable(signature);
    if (signature == "java.lang.Integer") {
        // Value is an integer
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jint(obj)))
    } else if (signature == "java.lang.Boolean") {
        // Value is a boolean
        TRY_RUN(return Napi::Boolean::New(env, j_env.jobject_to_jboolean(obj)))
    } else if (signature == "java.lang.Byte") {
        // Value is a byte
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jbyte(obj)))
    } else if (signature == "java.lang.Character") {
        // Value is a char
        TRY_RUN(return Napi::String::New(env, std::string(1, (char) j_env.jobject_to_jchar(obj))))
    } else if (signature == "java.lang.Short") {
        // Value is a short
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jshort(obj)))
    } else if (signature == "java.lang.Long") {
        // Value is a long
        // TODO: Return raw values
        TRY_RUN(return Napi::Number::New(env, (double) j_env.jobject_to_jlong(obj)))
    } else if (signature == "java.lang.Float") {
        // Value is a float
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jfloat(obj)))
    } else if (signature == "java.lang.Double") {
        // Value is a double
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jdouble(obj)))
    } else if (signature == "java.lang.String") {
        // Value is a string
        TRY_RUN(return Napi::String::New(env,
                                         j_env.jstring_to_string(reinterpret_cast<jstring>(obj.obj))))
    } else if (util::hasEnding(signature, "[]")) {
        // The value is an array
        try {
            jni::jobject_wrapper<jobjectArray> j_array = obj.as<jobjectArray>();
            const jsize size = j_env->GetArrayLength(j_array);
            Napi::Array array = Napi::Array::New(env, static_cast<size_t>(size));

            for (jsize i = 0; i < size; i++) {
                auto cur = jni::jobject_wrapper(j_env->GetObjectArrayElement(j_array, i), j_env);
                array.Set(i, jobject_to_value(env, cur, signature.substr(1)));
            }

            return array;
        } catch (const std::exception &e) {
            throw Napi::Error::New(env, e.what());
        }
    } else {
        // The value is a class instance
        Napi::Object class_proxy = node_classes::java::getClass(env, signature);

        return node_classes::java_instance_proxy::fromJObject(env, obj, class_proxy);
    }
}

#define CHECK_TYPE_MATCH(check, type) \
if (!value.check())                   \
    throw Napi::TypeError::New(env, "Expected type " #type " but got " + napi_valuetype_to_string(value.Type()))

jni::jobject_wrapper<jobject> conversion_helper::value_to_jobject(const Napi::Env &env, const Napi::Value &value,
                                                                  std::string signature) {
    if (value.IsNull()) {
        return jni::jobject_wrapper<jobject>();
    }

    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    signature = util::make_java_name_readable(signature);
    if (signature == "java.lang.Integer") {
        // Value is an integer
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jint(value.ToNumber().operator int()))
    } else if (signature == "java.lang.Boolean") {
        // Value is a boolean
        CHECK_TYPE_MATCH(IsBoolean, boolean);
        TRY_RUN(return j_env.create_jboolean(value.ToBoolean()))
    } else if (signature == "java.lang.Byte") {
        // Value is a byte
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jbyte((jbyte) value.ToNumber().operator int()))
    } else if (signature == "java.lang.Char") {
        // Value is a char
        CHECK_TYPE_MATCH(IsString, string);
        TRY_RUN(return j_env.create_jchar(value.ToString().Utf8Value()[0]))
    } else if (signature == "java.lang.Short") {
        // Value is a short
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jshort((jshort) value.ToNumber().operator int()))
    } else if (signature == "java.lang.Long") {
        // Value is a long
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jlong((jlong) value.ToNumber().operator long long()))
    } else if (signature == "java.lang.Float") {
        // Value is a float
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jfloat(value.ToNumber().operator float()))
    } else if (signature == "java.lang.Double") {
        // Value is a double
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jdouble(value.ToNumber().operator double()))
    } else if (signature == "java.lang.String") {
        // Value is a string
        CHECK_TYPE_MATCH(IsString, string);
        TRY_RUN(return j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>())
    } else if (util::hasEnding(signature, "[]")) {
        CHECK_TYPE_MATCH(IsArray, array);
        auto array = value.As<Napi::Array>();
        try {
            std::string classname = signature.substr(0, signature.size() - 2);
            jclass clazz = j_env.getJClass(classname);

            jint array_size = static_cast<jint>(array.Length());

            jni::jobject_wrapper<jobjectArray> j_array(j_env->NewObjectArray(array_size, clazz, nullptr), j_env);
            j_env.checkForError();

            for (jint i = 0; i < array_size; i++) {
                j_env->SetObjectArrayElement(j_array, i, value_to_jobject(env, array.Get(i), classname));
                j_env.checkForError();
            }

            return j_array.as<jobject>();
        } catch (const std::exception &e) {
            throw Napi::Error::New(env, e.what());
        }
    } else {
        // Expecting a class instance
        CHECK_TYPE_MATCH(IsObject, object);
        Napi::Object obj = value.ToObject();
        auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(obj);

        try {
            std::string classname = util::make_java_name_readable(ptr->classname);
            std::string expected_classname = util::make_java_name_readable(signature);

            if (!j_env.class_is_assignable(classname, expected_classname)) {
                throw Napi::TypeError::New(env, "Expected class " + expected_classname + " but got " + classname);
            }

            return ptr->object;
        } catch (const std::exception &e) {
            throw Napi::Error::New(env, e.what());
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

bool value_type_matches_signature(const Napi::Value &value, std::string signature, const jni::jni_wrapper &j_env) {
    signature = util::make_java_name_readable(signature);

    if (value.IsNull()) {
        return true;
    } else if (value.IsBoolean()) {
        return signature == "java.lang.Boolean";
    } else if (value.IsNumber()) {
        return signature == "java.lang.Byte" || signature == "java.lang.Short" || signature == "java.lang.Integer" ||
               signature == "java.lang.Long" || signature == "java.lang.Float" || signature == "java.lang.Double";
    } else if (value.IsString()) {
        return (!util::hasEnding(signature, "[]") && j_env.class_is_assignable("java.lang.String", signature)) ||
               (value.ToString().Utf8Value().size() == 1 && signature == "java.lang.Character");
    } else if (value.IsArray()) {
        if (util::hasEnding(signature, "[]")) {
            const auto array = value.As<Napi::Array>();
            if (array.Length() == 0) {
                return true;
            } else {
                const uint32_t zero = 0;
                return value_type_matches_signature(array.Get(zero), signature.substr(0, signature.size() - 2), j_env);
            }
        } else {
            return false;
        }
    } else if (value.IsObject()) {
        auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(value.ToObject());
        return j_env.class_is_assignable(ptr->classname, signature);
    } else {
        return false;
    }
}

bool args_match_java_types(const Napi::CallbackInfo &args, const std::vector<std::string> &parameterTypes,
                           const jni::jni_wrapper &j_env) {
    if (args.Length() != parameterTypes.size()) {
        return false;
    }

    for (size_t i = 0; i < parameterTypes.size(); i++) {
        if (!value_type_matches_signature(args[i], parameterTypes[i], j_env)) {
            return false;
        }
    }

    return true;
}

std::vector<jni::jobject_wrapper<jobject>> args_to_java_arguments(const Napi::CallbackInfo &args,
                                                                  const std::vector<std::string> &parameterTypes) {
    std::vector<jni::jobject_wrapper<jobject>> arguments;
    for (size_t i = 0; i < args.Length(); i++) {
        arguments.push_back(conversion_helper::value_to_jobject(args.Env(), args[i], parameterTypes[i]));
    }

    return arguments;
}

jni::jobject_wrapper<jobject> conversion_helper::match_constructor_arguments(const Napi::CallbackInfo &args,
                                                                             const std::vector<jni::java_constructor> &constructors) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    for (const auto &c : constructors) {
        if (args_match_java_types(args, c.parameterTypes, j_env)) {
            return c.newInstance(args_to_java_arguments(args, c.parameterTypes));
        }
    }

    std::stringstream ss;
    ss << "Could not find an appropriate constructor. Options were:";
    for (const auto &c : constructors) {
        ss << std::endl << '\t' << c.to_string();
    }

    throw Napi::TypeError::New(args.Env(), ss.str());
}

const jni::java_constructor *conversion_helper::find_matching_constructor(const Napi::CallbackInfo &args,
                                                                          const std::vector<jni::java_constructor> &constructors,
                                                                          std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                                          std::string &error) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    for (const auto &c : constructors) {
        if (args_match_java_types(args, c.parameterTypes, j_env)) {
            outArgs = args_to_java_arguments(args, c.parameterTypes);
            return &c;
        }
    }

    std::stringstream ss;
    ss << "Could not find an appropriate constructor. Options were:";
    for (const auto &c : constructors) {
        ss << std::endl << '\t' << c.to_string();
    }

    error = ss.str();
    return nullptr;
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
                          const Napi::Env &env, const std::vector<jvalue> &args, const jni::jni_wrapper &j_env) {
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
        return conversion_helper::jobject_to_value(env, obj, signature);
    }
}

#undef CALL_FUNCTION

#define CALL_STATIC_FUNCTION(functionName) auto res = j_env->functionName(clazz, function.method, args.data());\
                                            j_env.checkForError()

Napi::Value call_static_function(const jni::java_function &function, jclass clazz, const Napi::Env &env,
                                 const std::vector<jvalue> &args, const jni::jni_wrapper &j_env) {
    const std::string &signature = function.returnType;
    if (signature == "V") {
        // Method returns void
        j_env->CallStaticVoidMethodA(clazz, function.method, args.data());
        j_env.checkForError();
        return env.Undefined();
    } else if (signature == "I") {
        // Value is an integer
        CALL_STATIC_FUNCTION(CallStaticIntMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "Z") {
        // Value is a boolean
        CALL_STATIC_FUNCTION(CallStaticBooleanMethodA);
        return Napi::Boolean::New(env, res);
    } else if (signature == "B") {
        // Value is a byte
        CALL_STATIC_FUNCTION(CallStaticByteMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "C") {
        // Value is a char
        CALL_STATIC_FUNCTION(CallStaticCharMethodA);
        return Napi::String::New(env, std::string({static_cast<char>(res)}));
    } else if (signature == "S") {
        // Value is a short
        CALL_STATIC_FUNCTION(CallStaticShortMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "J") {
        // Value is a long
        CALL_STATIC_FUNCTION(CallStaticLongMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "F") {
        // Value is a float
        CALL_STATIC_FUNCTION(CallStaticFloatMethodA);
        return Napi::Number::From(env, res);
    } else if (signature == "D") {
        // Value is a double
        CALL_STATIC_FUNCTION(CallStaticDoubleMethodA);
        return Napi::Number::From(env, res);
    } else {
        // Value is some kind of object
        CALL_STATIC_FUNCTION(CallStaticObjectMethodA);
        jni::jobject_wrapper<jobject> obj(res, j_env);
        return conversion_helper::jobject_to_value(env, obj, signature);
    }
}

#undef CALL_STATIC_FUNCTION

Napi::TypeError throw_no_matching_fn(const Napi::Env &env, const std::vector<jni::java_function> &functions) {
    std::stringstream ss;
    ss << "Could not find a matching function. Options were:";
    for (const auto &f : functions) {
        ss << std::endl << '\t' << f.to_string();
    }

    return Napi::TypeError::New(env, ss.str());
}

Napi::Value conversion_helper::call_matching_function(const Napi::CallbackInfo &args,
                                                      const jni::jobject_wrapper<jobject> &classInstance,
                                                      const std::vector<jni::java_function> &functions) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    for (const auto &f : functions) {
        if (args_match_java_types(args, f.parameterTypes, j_env)) {
            std::vector<jni::jobject_wrapper<jobject>> j_args = args_to_java_arguments(args, f.parameterTypes);
            std::vector<jvalue> values = jobjects_to_jvalues(f.parameterTypes, j_env, j_args);

            return call_function(f, classInstance, args.Env(), values, j_env);
        }
    }

    throw throw_no_matching_fn(args.Env(), functions);
}

Napi::Value conversion_helper::call_matching_static_function(const Napi::CallbackInfo &args, jclass clazz,
                                                             const std::vector<jni::java_function> &functions) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();

    for (const auto &f : functions) {
        if (args_match_java_types(args, f.parameterTypes, j_env)) {
            std::vector<jni::jobject_wrapper<jobject>> j_args = args_to_java_arguments(args, f.parameterTypes);
            std::vector<jvalue> values = jobjects_to_jvalues(f.parameterTypes, j_env, j_args);

            return call_static_function(f, clazz, args.Env(), values, j_env);
        }
    }

    throw throw_no_matching_fn(args.Env(), functions);
}

const jni::java_function *conversion_helper::find_matching_function(const Napi::CallbackInfo &args,
                                                                    const std::vector<jni::java_function> &functions,
                                                                    std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                                    std::string &error,
                                                                    std::vector<jvalue> &outValues) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    for (const auto &f : functions) {
        if (args_match_java_types(args, f.parameterTypes, j_env)) {
            outArgs = args_to_java_arguments(args, f.parameterTypes);
            outValues = jobjects_to_jvalues(f.parameterTypes, j_env, outArgs);

            return &f;
        }
    }

    std::stringstream ss;
    ss << "Could not find a matching function. Options were:";
    for (const auto &f : functions) {
        ss << std::endl << '\t' << f.to_string();
    }
    error = ss.str();
    return nullptr;
}

#define CALL_FUNCTION(functionName) auto res = j_env->functionName(classInstance, function.method, args.data()); \
                                    j_env.checkForError()

jvalue conversion_helper::call_function(const jni::java_function &function,
                                        const jni::jobject_wrapper<jobject> &classInstance,
                                        const std::vector<jvalue> &args) {
    const std::string &signature = function.returnType;
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();

    jvalue val;
    if (signature == "V") {
        // Method returns void
        j_env->CallVoidMethodA(classInstance, function.method, args.data());
        j_env.checkForError();
    } else if (signature == "I") {
        // Value is an integer
        CALL_FUNCTION(CallIntMethodA);
        val.i = res;
    } else if (signature == "Z") {
        // Value is a boolean
        CALL_FUNCTION(CallBooleanMethodA);
        val.z = res;
    } else if (signature == "B") {
        // Value is a byte
        CALL_FUNCTION(CallByteMethodA);
        val.b = res;
    } else if (signature == "C") {
        // Value is a char
        CALL_FUNCTION(CallCharMethodA);
        val.c = res;
    } else if (signature == "S") {
        // Value is a short
        CALL_FUNCTION(CallShortMethodA);
        val.s = res;
    } else if (signature == "J") {
        // Value is a long
        CALL_FUNCTION(CallLongMethodA);
        val.j = res;
    } else if (signature == "F") {
        // Value is a float
        CALL_FUNCTION(CallFloatMethodA);
        val.f = res;
    } else if (signature == "D") {
        // Value is a double
        CALL_FUNCTION(CallDoubleMethodA);
        val.d = res;
    } else {
        // Value is some kind of object
        CALL_FUNCTION(CallObjectMethodA);
        val.l = j_env->NewGlobalRef(res);
    }

    return val;
}

#undef CALL_FUNCTION

#define CALL_STATIC_FUNCTION(functionName) auto res = j_env->functionName(clazz, function.method, args.data());\
                                            j_env.checkForError()

jvalue conversion_helper::call_static_function(const jni::java_function &function, jclass clazz,
                                               const std::vector<jvalue> &args) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    const std::string &signature = function.returnType;

    jvalue val;
    if (signature == "V") {
        // Method returns void
        j_env->CallStaticVoidMethodA(clazz, function.method, args.data());
        j_env.checkForError();
    } else if (signature == "I") {
        // Value is an integer
        CALL_STATIC_FUNCTION(CallStaticIntMethodA);
        val.i = res;
    } else if (signature == "Z") {
        // Value is a boolean
        CALL_STATIC_FUNCTION(CallStaticBooleanMethodA);
        val.z = res;
    } else if (signature == "B") {
        // Value is a byte
        CALL_STATIC_FUNCTION(CallStaticByteMethodA);
        val.b = res;
    } else if (signature == "C") {
        // Value is a char
        CALL_STATIC_FUNCTION(CallStaticCharMethodA);
        val.c = res;
    } else if (signature == "S") {
        // Value is a short
        CALL_STATIC_FUNCTION(CallStaticShortMethodA);
        val.s = res;
    } else if (signature == "J") {
        // Value is a long
        CALL_STATIC_FUNCTION(CallStaticLongMethodA);
        val.j = res;
    } else if (signature == "F") {
        // Value is a float
        CALL_STATIC_FUNCTION(CallStaticFloatMethodA);
        val.f = res;
    } else if (signature == "D") {
        // Value is a double
        CALL_STATIC_FUNCTION(CallStaticDoubleMethodA);
        val.d = res;
    } else {
        // Value is some kind of object
        CALL_STATIC_FUNCTION(CallStaticObjectMethodA);
        val.l = j_env->NewGlobalRef(res);
    }

    return val;
}

#undef CALL_STATIC_FUNCTION

Napi::Value conversion_helper::jvalue_to_napi_value(jvalue value, const std::string &signature, const Napi::Env &env) {
    if (signature == "V") {
        // Method returns void
        return env.Undefined();
    } else if (signature == "I") {
        // Value is an integer
        return Napi::Number::From(env, value.i);
    } else if (signature == "Z") {
        // Value is a boolean
        return Napi::Boolean::New(env, value.z);
    } else if (signature == "B") {
        // Value is a byte
        return Napi::Number::From(env, value.b);
    } else if (signature == "C") {
        // Value is a char
        return Napi::String::New(env, std::string({static_cast<char>(value.c)}));
    } else if (signature == "S") {
        // Value is a short
        return Napi::Number::From(env, value.s);
    } else if (signature == "J") {
        // Value is a long
        return Napi::Number::From(env, value.j);
    } else if (signature == "F") {
        // Value is a float
        return Napi::Number::From(env, value.f);
    } else if (signature == "D") {
        // Value is a double
        return Napi::Number::From(env, value.d);
    } else {
        // Value is some kind of object
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
        jni::jobject_wrapper<jobject> obj(value.l, j_env);

        j_env->DeleteGlobalRef(value.l);
        return conversion_helper::jobject_to_value(env, obj, signature);
    }
}