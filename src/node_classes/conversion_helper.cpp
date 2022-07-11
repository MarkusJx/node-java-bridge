#include "node_classes/conversion_helper.hpp"
#include "node_classes/java_function_caller.hpp"
#include "node_classes/java_instance_proxy.hpp"
#include "node_classes/jvm_container.hpp"
#include "node_classes/java.hpp"
#include "util/util.hpp"

#ifdef ENABLE_LOGGING
#   include <logger.hpp>
#endif //ENABLE_LOGGING

#define TRY_RUN(statement) try { \
                                statement;\
                            } catch (const std::exception &e) {\
                                throw Napi::Error::New(env, e.what()); \
                            }

java_type get_object_type(const jni::jni_wrapper &j_env, const java_type &signature,
                          const jni::jobject_wrapper<jobject> &obj) {
    if (obj.isNull()) {
        throw RUNTIME_ERROR("The object was null");
    }

    if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Integer"))) {
        return java_type(j_type::lang_Integer, nullptr, "java.lang.Integer");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Boolean"))) {
        return java_type(j_type::lang_Boolean, nullptr, "java.lang.Boolean");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Byte"))) {
        return java_type(j_type::lang_Byte, nullptr, "java.lang.Byte");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Character"))) {
        return java_type(j_type::lang_Character, nullptr, "java.lang.Character");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Short"))) {
        return java_type(j_type::lang_Short, nullptr, "java.lang.Short");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Long"))) {
        return java_type(j_type::lang_Long, nullptr, "java.lang.Long");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Float"))) {
        return java_type(j_type::lang_Float, nullptr, "java.lang.Float");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.Double"))) {
        return java_type(j_type::lang_Double, nullptr, "java.lang.Double");
    } else if (j_env->IsInstanceOf(obj, j_env.getJClass("java.lang.String"))) {
        return java_type(j_type::String, nullptr, "java.lang.String");
    } else if (signature.isArray()) {
        return java_type(j_type::Array, std::make_shared<java_type>(get_object_type(j_env, *signature.inner, obj)),
                         signature.signature);
    } else if (signature == j_type::lang_Object) {
        return java_type::to_java_type(j_env.getObjectClassName(obj));
    } else {
        return signature;
    }
}

template<class T, class R>
Napi::Array create_array(const Napi::Env &env, const jni::jobject_wrapper<jobject> &obj, const jni::jni_wrapper &j_env,
                         const std::function<R *(T)> &get_array_elements,
                         const std::function<Napi::Value(R)> &get_array_element,
                         const std::function<void(const jni::jobject_wrapper<T> &, R *)> &releaser) {
    jni::jobject_wrapper<T> j_array = obj.as<T>();
    const jsize size = j_env->GetArrayLength(j_array);
    j_env.checkForError();
    Napi::Array array = Napi::Array::New(env, static_cast<size_t>(size));
    R *arr = get_array_elements(j_array);
    j_env.checkForError();

    for (jsize i = 0; i < size; i++) {
        array.Set(i, get_array_element(arr[i]));
        j_env.checkForError();
    }

    releaser(j_array, arr);
    j_env.checkForError();
    return array;
}

Napi::Value conversion_helper::jobject_to_value(const Napi::Env &env, const jni::jobject_wrapper<jobject> &obj,
                                                const java_type &signature, bool objects) {
    if (obj.isNull()) {
        return env.Null();
    }

    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    if (objects && signature == j_type::lang_Object) {
        TRY_RUN(return jobject_to_value(env, obj, get_object_type(j_env, signature, obj), false))
    } else if (signature == j_type::lang_Integer || signature.isInt()) {
        // Value is an integer
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jint(obj)))
    } else if (signature == j_type::lang_Boolean || signature.isBool()) {
        // Value is a boolean
        TRY_RUN(return Napi::Boolean::New(env, j_env.jobject_to_jboolean(obj)))
    } else if (signature == j_type::lang_Byte || signature.isByte()) {
        // Value is a byte
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jbyte(obj)))
    } else if (signature == j_type::lang_Character || signature.isChar()) {
        // Value is a char
        TRY_RUN(return Napi::String::New(env, std::string(1, (char) j_env.jobject_to_jchar(obj))))
    } else if (signature == j_type::lang_Short || signature.isShort()) {
        // Value is a short
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jshort(obj)))
    } else if (signature == j_type::lang_Long || signature.isLong()) {
        // Value is a long
        TRY_RUN(return Napi::BigInt::New(env, int64_t(j_env.jobject_to_jlong(obj))))
    } else if (signature == j_type::lang_Float || signature.isFloat()) {
        // Value is a float
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jfloat(obj)))
    } else if (signature == j_type::lang_Double || signature.isDouble()) {
        // Value is a double
        TRY_RUN(return Napi::Number::New(env, j_env.jobject_to_jdouble(obj)))
    } else if (signature == j_type::String) {
        // Value is a string
        TRY_RUN(return Napi::String::New(env,
                                         j_env.jstring_to_string(reinterpret_cast<jstring>(obj.obj))))
    } else if (signature.isArray()) {
        // The value is an array
        try {
            switch (signature.inner->type) {
                case Integer:
                    return create_array<jintArray, jint>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetIntArrayElements(arr, nullptr);
                    }, [&env](jint el) {
                        return Napi::Number::From(env, el);
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseIntArrayElements(a1, a2, JNI_ABORT);
                    });
                case Boolean:
                    return create_array<jbooleanArray, jboolean>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetBooleanArrayElements(arr, nullptr);
                    }, [&env](jboolean el) {
                        return Napi::Boolean::New(env, static_cast<bool>(el));
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseBooleanArrayElements(a1, a2, JNI_ABORT);
                    });
                case Byte: {
                    jni::jobject_wrapper<jbyteArray> j_array = obj.as<jbyteArray>();
                    jbyte *elements = j_env->GetByteArrayElements(j_array, nullptr);
                    j_env.checkForError();
                    jsize length = j_env->GetArrayLength(j_array);
                    j_env.checkForError();

                    const auto releaser = [j_array](const Napi::Env &, jbyte *arr) {
                        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
                        j_env->ReleaseByteArrayElements(j_array, arr, JNI_ABORT);
                        j_env.checkForError();
                    };

                    return Napi::Buffer<jbyte>::New(env, elements, static_cast<size_t>(length), releaser);
                }
                case Character:
                    return create_array<jcharArray, jchar>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetCharArrayElements(arr, nullptr);
                    }, [&env](jchar el) {
                        return Napi::String::New(env, std::u16string(1, static_cast<char16_t>(el)));
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseCharArrayElements(a1, a2, JNI_ABORT);
                    });
                case Short:
                    return create_array<jshortArray, jshort>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetShortArrayElements(arr, nullptr);
                    }, [&env](jshort el) {
                        return Napi::Number::From(env, el);
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseShortArrayElements(a1, a2, JNI_ABORT);
                    });
                case Long:
                    return create_array<jlongArray, jlong>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetLongArrayElements(arr, nullptr);
                    }, [&env](jlong el) {
                        return Napi::Number::From(env, el);
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseLongArrayElements(a1, a2, JNI_ABORT);
                    });
                case Float:
                    return create_array<jfloatArray, jfloat>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetFloatArrayElements(arr, nullptr);
                    }, [&env](jfloat el) {
                        return Napi::Number::From(env, el);
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseFloatArrayElements(a1, a2, JNI_ABORT);
                    });
                case Double:
                    return create_array<jdoubleArray, jdouble>(env, obj, j_env, [&j_env](auto arr) {
                        return j_env->GetDoubleArrayElements(arr, nullptr);
                    }, [&env](jdouble el) {
                        return Napi::Number::From(env, el);
                    }, [&j_env](auto a1, auto a2) {
                        j_env->ReleaseDoubleArrayElements(a1, a2, JNI_ABORT);
                    });
                default: {
                    jni::jobject_wrapper<jobjectArray> j_array = obj.as<jobjectArray>();
                    const jsize size = j_env->GetArrayLength(j_array);
                    j_env.checkForError();
                    Napi::Array array = Napi::Array::New(env, static_cast<size_t>(size));

                    for (jsize i = 0; i < size; i++) {
                        auto cur = jni::jobject_wrapper(j_env->GetObjectArrayElement(j_array, i), j_env.env);
                        array.Set(i, jobject_to_value(env, cur, *signature.inner));
                    }

                    return array;
                }
            }
        } catch (const std::exception &e) {
            throw Napi::Error::New(env, e.what());
        }
    } else {
        // The value is a class instance
        Napi::Object class_proxy = node_classes::java::getClass(env, signature.signature);

        return node_classes::java_instance_proxy::fromJObject(env, obj, class_proxy);
    }
}

#define CHECK_TYPE_MATCH(check, type) \
if (!value.check())                   \
    throw Napi::TypeError::New(env, __FILE__ ":" + std::to_string(__LINE__) + " Expected type " #type " but got " + \
                                napi_valuetype_to_string(value.Type()))

jni::jobject_wrapper<jobject> conversion_helper::value_to_jobject(const Napi::Env &env, const Napi::Value &value,
                                                                  const java_type &signature, bool objects) {
    if (value.IsNull()) {
        return {};
    }

    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
#ifdef ENABLE_LOGGING
    markusjx::logging::StaticLogger::debugStream << "Converting value of type "
                                                 << napi_valuetype_to_string(value.Type()) << " to java object type "
                                                 << signature.signature;
#endif //ENABLE_LOGGING

    if (objects && signature == j_type::lang_Object) {
        // If the function accepts objects, try to match the type by the passed js type
        if (value.IsNumber()) {
            if (is_integer(env, value.ToNumber())) {
                TRY_RUN(return j_env.create_jint(value.ToNumber().Int32Value()))
            } else {
                TRY_RUN(return j_env.create_jdouble(value.ToNumber().DoubleValue()))
            }
        } else if (value.IsBigInt()) {
            bool lossless;
            TRY_RUN(return j_env.create_jlong(value.As<Napi::BigInt>().Int64Value(&lossless)))
        } else if (value.IsBoolean()) {
            TRY_RUN(return j_env.create_jboolean(value.ToBoolean().Value()))
        } else if (value.IsString()) {
            TRY_RUN(return j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>())
        } else if (value.IsArray()) {
            auto array = value.As<Napi::Array>();
            try {
                jclass clazz = j_env.getJClass("java.lang.Object");
                jint array_size = static_cast<jint>(array.Length());

                jni::jobject_wrapper<jobjectArray> j_array(j_env->NewObjectArray(array_size, clazz, nullptr), j_env.env);
                j_env.checkForError();

                for (jint i = 0; i < array_size; i++) {
                    j_env->SetObjectArrayElement(j_array, i, value_to_jobject(env, array.Get(i), signature, objects));
                    j_env.checkForError();
                }

                return j_array.as<jobject>();
            } catch (const std::exception &e) {
                throw Napi::Error::New(env, e.what());
            }
        } else {
            Napi::Object obj = value.ToObject();
            if (node_classes::java_function_caller::instanceOf(obj)) {
                return Napi::ObjectWrap<node_classes::java_function_caller>::Unwrap(obj)->proxy;
            } else {
                auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(obj);
                TRY_RUN(return ptr->object)
            }
        }
    } else if (signature == j_type::lang_Integer || signature.isInt()) {
        // Value is an integer
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jint(value.ToNumber().Int32Value()))
    } else if (signature == j_type::lang_Boolean || signature.isBool()) {
        // Value is a boolean
        CHECK_TYPE_MATCH(IsBoolean, boolean);
        TRY_RUN(return j_env.create_jboolean(value.ToBoolean().Value()))
    } else if (signature == j_type::lang_Byte || signature.isByte()) {
        // Value is a byte
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jbyte((jbyte) value.ToNumber().Int32Value()))
    } else if (signature == j_type::lang_Character || signature.isChar()) {
        // Value is a char
        CHECK_TYPE_MATCH(IsString, string);
        TRY_RUN(return j_env.create_jchar(value.ToString().Utf16Value()[0]))
    } else if (signature == j_type::lang_Short || signature.isShort()) {
        // Value is a short
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jshort((jshort) value.ToNumber().Int32Value()))
    } else if (signature == j_type::lang_Long || signature.isLong()) {
        // Value is a long
        if (!value.IsNumber() && !value.IsBigInt()) {
            throw Napi::TypeError::New(env, "Expected type number but got " + napi_valuetype_to_string(value.Type()));
        }

        if (value.IsNumber()) {
            TRY_RUN(return j_env.create_jlong((jlong) value.ToNumber().Int64Value()))
        } else {
            bool lossless;
            TRY_RUN(return j_env.create_jlong(value.As<Napi::BigInt>().Int64Value(&lossless)))
        }
    } else if (signature == j_type::lang_Float || signature.isFloat()) {
        // Value is a float
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jfloat(value.ToNumber().FloatValue()))
    } else if (signature == j_type::lang_Double || signature.isDouble()) {
        // Value is a double
        CHECK_TYPE_MATCH(IsNumber, number);
        TRY_RUN(return j_env.create_jdouble(value.ToNumber().DoubleValue()))
    } else if (j_env.class_is_assignable("java.lang.String", signature.signature) && signature != j_type::lang_Object) {
        // Value is a string
        CHECK_TYPE_MATCH(IsString, string);
        TRY_RUN(return j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>())
    } else if (signature.isArray()) {
        CHECK_TYPE_MATCH(IsArray, array);
        auto array = value.As<Napi::Array>();
        try {
            std::string classname = signature.inner->signature;
            jclass clazz = j_env.getJClass(classname);

            jint array_size = static_cast<jint>(array.Length());

            jni::jobject_wrapper<jobjectArray> j_array(j_env->NewObjectArray(array_size, clazz, nullptr), j_env.env);
            j_env.checkForError();

            for (jint i = 0; i < array_size; i++) {
                j_env->SetObjectArrayElement(j_array, i,
                                             value_to_jobject(env, array.Get(i), *signature.inner, objects));
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
        if (node_classes::java_function_caller::instanceOf(obj)) {
            return Napi::ObjectWrap<node_classes::java_function_caller>::Unwrap(obj)->proxy;
        }

        auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(obj);

        try {
            std::string classname = util::make_java_name_readable(ptr->classname);
            const std::string &expected_classname = signature.signature;

            if (!j_env.class_is_assignable(classname, expected_classname)) {
                throw Napi::TypeError::New(env, "Expected class " + expected_classname + " but got " + classname);
            }

            return ptr->object;
        } catch (const std::exception &e) {
            throw Napi::Error::New(env, e.what());
        }
    }
}

jvalue conversion_helper::napi_value_to_jvalue(const Napi::Env &env, const Napi::Value &value,
                                               const java_type &signature,
                                               std::vector<jni::jobject_wrapper<jobject>> &values, bool objects) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
#ifdef ENABLE_LOGGING
    markusjx::logging::StaticLogger::debugStream << "Converting value of type "
                                                 << conversion_helper::napi_valuetype_to_string(value.Type())
                                                 << " to java type " << signature.signature;
#endif //ENABLE_LOGGING

    jvalue val;
    if (objects && signature == j_type::lang_Object) {
        auto object = value_to_jobject(env, value, signature, objects);
        values.push_back(object);
        val.l = object;
        return val;
    } else if (signature == j_type::Integer) {
        // Value is an integer
        CHECK_TYPE_MATCH(IsNumber, number);
        val.i = value.ToNumber().Int32Value();
    } else if (signature == j_type::Boolean) {
        // Value is a boolean
        CHECK_TYPE_MATCH(IsBoolean, boolean);
        val.z = value.ToBoolean().Value();
    } else if (signature == j_type::Byte) {
        // Value is a byte
        CHECK_TYPE_MATCH(IsNumber, number);
        val.b = static_cast<jbyte>(value.ToNumber().Int32Value());
    } else if (signature == j_type::Character) {
        // Value is a char
        CHECK_TYPE_MATCH(IsString, string);
        val.c = value.ToString().Utf16Value()[0];
    } else if (signature == j_type::Short) {
        // Value is a short
        CHECK_TYPE_MATCH(IsNumber, number);
        val.s = static_cast<jshort>(value.ToNumber().Int32Value());
    } else if (signature == j_type::Long) {
        // Value is a long
        if (!value.IsNumber() && !value.IsBigInt())
            throw Napi::TypeError::New(env, __FILE__ ":" + std::to_string(__LINE__) +
                                            "Expected type number but got " + napi_valuetype_to_string(value.Type()));

        if (value.IsNumber()) {
            val.j = static_cast<jlong>(value.ToNumber().Int64Value());
        } else {
            bool lossless;
            val.j = value.As<Napi::BigInt>().Int64Value(&lossless);
        }
    } else if (signature == j_type::Float) {
        // Value is a float
        CHECK_TYPE_MATCH(IsNumber, number);
        val.f = static_cast<jfloat>(value.ToNumber().FloatValue());
    } else if (signature == j_type::Double) {
        // Value is a double
        CHECK_TYPE_MATCH(IsNumber, number);
        val.d = static_cast<jdouble>(value.ToNumber().DoubleValue());
    } else if (j_env.class_is_assignable("java.lang.String", signature.signature) && signature != j_type::lang_Object) {
        // Value is a string
        if (!value.IsString() && !value.IsNull()) {
            throw Napi::TypeError::New(env, __FILE__ ":" + std::to_string(__LINE__) +
                                            " Expected type string but got " + napi_valuetype_to_string(value.Type()));
        }

        if (value.IsNull()) {
            val.l = nullptr;
        } else {
            auto object = j_env.string_to_jstring(value.ToString().Utf8Value()).as<jobject>();
            values.push_back(object);
            val.l = object.obj;
        }
    } else if (signature.isArray()) {
        // Expecting an array
        if (!value.IsArray() && !value.IsNull()) {
            throw Napi::TypeError::New(env, __FILE__ ":" + std::to_string(__LINE__) +
                                            " Expected type array but got " + napi_valuetype_to_string(value.Type()));
        }

        if (value.IsNull()) {
            val.l = nullptr;
        } else {
            auto array = value.As<Napi::Array>();
            try {
                auto object = napi_array_to_jarray(env, j_env, *signature.inner, array, objects);
                values.push_back(object.as<jobject>());
                val.l = object.obj;
            } catch (const std::exception &e) {
                throw Napi::Error::New(env, e.what());
            }
        }
    } else {
        // Expecting a class instance
        if (!value.IsObject() && !value.IsNull()) {
            throw Napi::TypeError::New(env, __FILE__ ":" + std::to_string(__LINE__) +
                                            " Expected type object but got " + napi_valuetype_to_string(value.Type()));
        }

        if (value.IsNull()) {
            val.l = nullptr;
        } else {
            node_classes::java_instance_proxy *ptr;
            try {
                ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(value.ToObject());
            } catch (...) {
                auto object = value_to_jobject(env, value, signature, objects);
                values.push_back(object);
                val.l = object;
                return val;
            }

            try {
                std::string classname = util::make_java_name_readable(ptr->classname);
                const std::string &expected_classname = signature.signature;

                if (!j_env.class_is_assignable(classname, expected_classname)) {
                    throw Napi::TypeError::New(env, "Expected class " + expected_classname + " but got " + classname);
                }

                values.push_back(ptr->object);
                val.l = ptr->object.obj;
            } catch (const std::exception &e) {
                throw Napi::Error::New(env, e.what());
            }
        }
    }

    return val;
}

#define POPULATE_ARRAY(T, U, creator, converter, setter)\
    const auto arrLen = static_cast<jsize>(array.Length());\
    jni::jobject_wrapper<jarray> res(j_env->creator(arrLen), j_env.env); \
    j_env.checkForError();\
    std::vector<U> values(array.Length());\
    for (jsize i = 0; i < arrLen; i++) {\
        values[i] = (U) array.Get(i).converter;\
    }\
\
    j_env->setter((T) res.obj, 0, arrLen, values.data());  \
    j_env.checkForError();\
    return res

jni::jobject_wrapper<jarray> conversion_helper::napi_array_to_jarray(const Napi::Env &env,
                                                                     const jni::jni_wrapper &j_env,
                                                                     const java_type &signature,
                                                                     const Napi::Array &array, bool objects) {
    if (signature == j_type::Integer) {
        // Value is an integer
        POPULATE_ARRAY(jintArray, jint, NewIntArray, ToNumber().Int32Value(), SetIntArrayRegion);
    } else if (signature == j_type::Boolean) {
        // Value is a boolean
        POPULATE_ARRAY(jbooleanArray, jboolean, NewBooleanArray, ToBoolean().Value(), SetBooleanArrayRegion);
    } else if (signature == j_type::Byte) {
        // Value is a byte
        POPULATE_ARRAY(jbyteArray, jbyte, NewByteArray, ToNumber().Int32Value(), SetByteArrayRegion);
    } else if (signature == j_type::Character) {
        // Value is a char
        POPULATE_ARRAY(jcharArray, jchar, NewCharArray, ToString().Utf16Value()[0], SetCharArrayRegion);
    } else if (signature == j_type::Short) {
        // Value is a short
        POPULATE_ARRAY(jshortArray, jshort, NewShortArray, ToNumber().Int32Value(), SetShortArrayRegion);
    } else if (signature == j_type::Long) {
        // Value is a long
        if (array.Length() > 0 && array.Get(uint32_t(0)).IsNumber()) {
            POPULATE_ARRAY(jlongArray, jlong, NewLongArray, ToNumber().Int64Value(), SetLongArrayRegion);
        } else {
            bool lossless;
            POPULATE_ARRAY(jlongArray, jlong, NewLongArray, As<Napi::BigInt>().Int64Value(&lossless),
                           SetLongArrayRegion);
        }
    } else if (signature == j_type::Float) {
        // Value is a float
        POPULATE_ARRAY(jfloatArray, jfloat, NewFloatArray, ToNumber().FloatValue(), SetFloatArrayRegion);
    } else if (signature == j_type::Double) {
        // Value is a double
        POPULATE_ARRAY(jdoubleArray, jdouble, NewDoubleArray, ToNumber().DoubleValue(), SetDoubleArrayRegion);
    } else if (j_env.class_is_assignable("java.lang.String", signature.signature) && signature != j_type::lang_Object) {
        // Value is a string
        const auto arrLen = static_cast<jsize>(array.Length());
        auto clazz = j_env.getJClass(signature.signature);
        jni::jobject_wrapper<jarray> res(j_env->NewObjectArray(arrLen, clazz, nullptr), j_env.env);
        j_env.checkForError();
        for (jsize i = 0; i < arrLen; i++) {
            j_env->SetObjectArrayElement((jobjectArray) res.obj, i, j_env.string_to_jstring(array.Get(i).ToString()));
            j_env.checkForError();
        }

        return res;
    } else if (signature.isArray()) {
        // Expecting an array-array
        const auto arrLen = static_cast<jsize>(array.Length());
        jclass clazz = j_env->FindClass(util::java_type_to_jni_type(signature.signature).substr(1).c_str());
        jni::jobject_wrapper<jarray> res(j_env->NewObjectArray(arrLen, clazz, nullptr), j_env.env);
        j_env.checkForError();

        std::vector<jni::jobject_wrapper<jobject>> tmp;
        for (jsize i = 0; i < arrLen; i++) {
            if (array.Get(uint32_t(i)).IsNull()) {
                j_env->SetObjectArrayElement((jobjectArray) res.obj, i, nullptr);
            } else {
                j_env->SetObjectArrayElement((jobjectArray) res.obj, i,
                                             napi_array_to_jarray(env, j_env, *signature.inner,
                                                                  array.Get(i).As<Napi::Array>(), objects).obj);
            }

            j_env.checkForError();
        }

        return res;
    } else {
        const auto arrLen = static_cast<jsize>(array.Length());
        auto clazz = j_env.getJClass(signature.signature);
        jni::jobject_wrapper<jarray> res(j_env->NewObjectArray(arrLen, clazz, nullptr), j_env.env);
        j_env.checkForError();
        std::vector<jni::jobject_wrapper<jobject>> tmp;
        for (jsize i = 0; i < arrLen; i++) {
            j_env->SetObjectArrayElement((jobjectArray) res.obj, i,
                                         napi_value_to_jvalue(env, array.Get(i), signature, tmp, objects).l);
            j_env.checkForError();
        }

        return res;
    }
}

#undef POPULATE_ARRAY

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

bool value_type_matches_signature(const Napi::Value &value, const java_type &signature,
                                  const jni::jni_wrapper &j_env, bool objects) {
    if (objects && signature == j_type::lang_Object) {
        // If passing anything as an object is allowed,
        // and this takes an object as an argument, this matches
        return true;
    } else if (value.IsNull()) {
        return (!signature.isArray() && j_env.class_is_assignable("java.lang.String", signature.signature)) ||
               signature.isArray() || !signature.isPrimitive();
    } else if (value.IsBoolean()) {
        return signature == j_type::lang_Boolean || signature == j_type::Boolean;
    } else if (value.IsNumber()) {
        return signature == j_type::lang_Byte || signature == j_type::lang_Short || signature == j_type::lang_Integer ||
               signature == j_type::lang_Long || signature == j_type::lang_Float || signature == j_type::lang_Double ||
               signature == j_type::Byte || signature == j_type::Short || signature == j_type::Integer ||
               signature == j_type::Long || signature == j_type::Float || signature == j_type::Double;
    } else if (value.IsBigInt()) {
        return signature == j_type::Long || signature == j_type::lang_Long;
    } else if (value.IsString()) {
        return (signature != j_type::lang_Object &&
                j_env.class_is_assignable("java.lang.String", signature.signature)) ||
               (value.ToString().Utf8Value().size() == 1 && signature != j_type::lang_Object &&
                (j_env.class_is_assignable("java.lang.Character", signature.signature) ||
                 signature == j_type::Character));
    } else if (value.IsArray()) {
        if (signature.isArray()) {
            const auto array = value.As<Napi::Array>();
            if (array.Length() == 0) {
                return true;
            } else {
                const uint32_t zero = 0;
                return value_type_matches_signature(array.Get(zero), *signature.inner, j_env, objects);
            }
        } else {
            return false;
        }
    } else if (value.IsObject()) {
        if (node_classes::java_function_caller::instanceOf(value.ToObject())) {
            return !signature.isPrimitive() && signature != j_type::String && signature != j_type::lang_Byte &&
                   signature != j_type::lang_Short && signature != j_type::lang_Integer &&
                   signature != j_type::lang_Long && signature != j_type::lang_Float &&
                   signature != j_type::lang_Double && signature != j_type::lang_Boolean;
        } else {
            auto *ptr = Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(value.ToObject());
            return j_env.class_is_assignable(ptr->classname, signature.signature);
        }
    } else {
        return false;
    }
}

bool args_match_java_types(const Napi::CallbackInfo &args, const std::vector<java_type> &parameterTypes,
                           const jni::jni_wrapper &j_env, bool objects) {
    if (args.Length() != parameterTypes.size()) {
        return false;
    }

    for (size_t i = 0; i < parameterTypes.size(); i++) {
        if (!value_type_matches_signature(args[i], parameterTypes[i], j_env, objects)) {
            return false;
        }
    }

    return true;
}

std::vector<jvalue> args_to_java_arguments(const Napi::CallbackInfo &args,
                                           const std::vector<java_type> &parameterTypes,
                                           std::vector<jni::jobject_wrapper<jobject>> &values, bool objects) {
    std::vector<jvalue> arguments;
    for (size_t i = 0; i < args.Length(); i++) {
        arguments.push_back(conversion_helper::napi_value_to_jvalue(args.Env(), args[i], parameterTypes[i],
                                                                    values, objects));
    }

    return arguments;
}

/**
 * Check how many instances of 'java.lang.Object' a function takes
 *
 * @param parameterTypes the function parameter types
 * @return the number of occurrences of java.lang.Object
 */
uint32_t get_num_objects(const std::vector<java_type> &parameterTypes) {
    uint32_t sz = 0;
    for (const auto &t: parameterTypes) {
        if (t.type == j_type::lang_Object) {
            sz++;
        }
    }

    return sz;
}

std::string napi_value_to_string(const Napi::Value &value) {
    if (value.IsObject()) {
        if (node_classes::java_function_caller::instanceOf(value.ToObject())) {
            try {
                return Napi::ObjectWrap<node_classes::java_function_caller>::Unwrap(value.ToObject())->getClassName();
            } catch (...) {
                return "object";
            }
        } else {
            try {
                return Napi::ObjectWrap<node_classes::java_instance_proxy>::Unwrap(value.ToObject())->classname;
            } catch (...) {
                return "object";
            }
        }
    } else if (value.IsArray()) {
        auto arr = value.As<Napi::Array>();
        if (arr.Length() == 0) {
            return "any[]";
        } else {
            return napi_value_to_string(arr.Get(uint32_t(0))) + "[]";
        }
    } else {
        return conversion_helper::napi_valuetype_to_string(value.Type());
    }
}

std::string js_args_to_string(const Napi::CallbackInfo &info) {
    std::stringstream ss;
    ss << '(';
    for (size_t i = 0; i < info.Length(); i++) {
        ss << napi_value_to_string(info[i]);

        if (i < info.Length() - 1) {
            ss << ", ";
        }
    }
    ss << ')';

    return ss.str();
}

const jni::java_constructor *conversion_helper::find_matching_constructor(const Napi::CallbackInfo &args,
                                                                          const std::vector<jni::java_constructor> &constructors,
                                                                          std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                                          std::string &error) {
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    const jni::java_constructor *generic_constructor = nullptr;
    uint32_t numObjects = 0;

    for (const auto &c: constructors) {
        if (args_match_java_types(args, c.parameterTypes, j_env, false)) {
            std::vector<jni::jobject_wrapper<jobject>> arguments;
            for (size_t i = 0; i < args.Length(); i++) {
                arguments.push_back(
                        conversion_helper::value_to_jobject(args.Env(), args[i], c.parameterTypes[i], false));
            }

            outArgs = arguments;
            return &c;
        } else if ((generic_constructor == nullptr || get_num_objects(c.parameterTypes) < numObjects) &&
                   args_match_java_types(args, c.parameterTypes, j_env, true)) {
            generic_constructor = &c;
            numObjects = get_num_objects(c.parameterTypes);
        }
    }

    // If we did not find a matching constructor but we
    // did find a matching generic constructor using
    // java.lang.Object, use that one
    if (generic_constructor != nullptr) {
        std::vector<jni::jobject_wrapper<jobject>> arguments;
        for (size_t i = 0; i < args.Length(); i++) {
            arguments.push_back(conversion_helper::value_to_jobject(args.Env(), args[i],
                                                                    generic_constructor->parameterTypes[i], true));
        }

        outArgs = arguments;
        return generic_constructor;
    }

    std::stringstream ss;
    ss << "Could not find an appropriate constructor with arguments: ";
    ss << js_args_to_string(args) << ". Options were:";
    for (const auto &c: constructors) {
        ss << std::endl << '\t' << c.to_string();
    }

    error = ss.str();
    return nullptr;
}

Napi::Value conversion_helper::call_matching_function(const Napi::CallbackInfo &info,
                                                      const jni::jobject_wrapper<jobject> &classInstance,
                                                      const std::vector<jni::java_function> &functions) {
    std::vector<jni::jobject_wrapper<jobject>> outArgs;
    std::string error;
    std::vector<jvalue> values;
    const jni::java_function *function = find_matching_function(info, functions, outArgs, error, values);

    if (function == nullptr) {
        throw Napi::TypeError::New(info.Env(), error);
    } else {
        TRY
            jvalue value = call_function(*function, classInstance, values);
            return jvalue_to_napi_value(value, function->returnType, info.Env());
        CATCH_EXCEPTIONS
    }
}

Napi::Value conversion_helper::call_matching_static_function(const Napi::CallbackInfo &info, jclass clazz,
                                                             const std::vector<jni::java_function> &functions) {
    std::vector<jni::jobject_wrapper<jobject>> outArgs;
    std::string error;
    std::vector<jvalue> values;
    const jni::java_function *function = find_matching_function(info, functions, outArgs, error, values);

    if (function == nullptr) {
        throw Napi::TypeError::New(info.Env(), error);
    } else {
        TRY
            jvalue value = call_static_function(*function, clazz, values);
            return jvalue_to_napi_value(value, function->returnType, info.Env());
        CATCH_EXCEPTIONS
    }
}

const jni::java_function *conversion_helper::find_matching_function(const Napi::CallbackInfo &args,
                                                                    const std::vector<jni::java_function> &functions,
                                                                    std::vector<jni::jobject_wrapper<jobject>> &outArgs,
                                                                    std::string &error,
                                                                    std::vector<jvalue> &outValues) {
    try {
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
        const jni::java_function *generic = nullptr;
        uint32_t numObjects = 0;

        for (const auto &f: functions) {
            if (args_match_java_types(args, f.parameterTypes, j_env, false)) {
                outValues = args_to_java_arguments(args, f.parameterTypes, outArgs, false);

                return &f;
            } else if ((generic == nullptr || get_num_objects(f.parameterTypes) < numObjects) &&
                       args_match_java_types(args, f.parameterTypes, j_env, true)) {
                generic = &f;
                numObjects = get_num_objects(f.parameterTypes);
            }
        }

        // If we did find a function which takes java.lang.Object as types
        // use that instead of throwing an exception
        if (generic != nullptr) {
            outValues = args_to_java_arguments(args, generic->parameterTypes, outArgs, true);
            return generic;
        }
    } catch (const std::exception &e) {
        error = e.what();
        return nullptr;
    }

    std::stringstream ss;
    ss << "Could not find a matching function with arguments: ";
    ss << js_args_to_string(args) << ". Options were:";
    for (const auto &f: functions) {
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
    if (classInstance.isNull()) {
        throw RUNTIME_ERROR("The class instance was null");
    }

    const java_type &signature = function.returnType;
    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    j_env.checkForError();

    jvalue val;
    if (signature.isVoid()) {
        // Method returns void
        j_env->CallVoidMethodA(classInstance, function.method, args.data());
        j_env.checkForError();
        val.l = nullptr;
    } else if (signature.isInt()) {
        // Value is an integer
        CALL_FUNCTION(CallIntMethodA);
        val.i = res;
    } else if (signature.isBool()) {
        // Value is a boolean
        CALL_FUNCTION(CallBooleanMethodA);
        val.z = res;
    } else if (signature.isByte()) {
        // Value is a byte
        CALL_FUNCTION(CallByteMethodA);
        val.b = res;
    } else if (signature.isChar()) {
        // Value is a char
        CALL_FUNCTION(CallCharMethodA);
        val.c = res;
    } else if (signature.isShort()) {
        // Value is a short
        CALL_FUNCTION(CallShortMethodA);
        val.s = res;
    } else if (signature.isLong()) {
        // Value is a long
        CALL_FUNCTION(CallLongMethodA);
        val.j = res;
    } else if (signature.isFloat()) {
        // Value is a float
        CALL_FUNCTION(CallFloatMethodA);
        val.f = res;
    } else if (signature.isDouble()) {
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
    if (clazz == nullptr) {
        throw RUNTIME_ERROR("The class pointer was null");
    }

    jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
    const java_type &signature = function.returnType;

    jvalue val;
    if (signature.isVoid()) {
        // Method returns void
        j_env->CallStaticVoidMethodA(clazz, function.method, args.data());
        j_env.checkForError();
        val.l = nullptr;
    } else if (signature.isInt()) {
        // Value is an integer
        CALL_STATIC_FUNCTION(CallStaticIntMethodA);
        val.i = res;
    } else if (signature.isBool()) {
        // Value is a boolean
        CALL_STATIC_FUNCTION(CallStaticBooleanMethodA);
        val.z = res;
    } else if (signature.isByte()) {
        // Value is a byte
        CALL_STATIC_FUNCTION(CallStaticByteMethodA);
        val.b = res;
    } else if (signature.isChar()) {
        // Value is a char
        CALL_STATIC_FUNCTION(CallStaticCharMethodA);
        val.c = res;
    } else if (signature.isShort()) {
        // Value is a short
        CALL_STATIC_FUNCTION(CallStaticShortMethodA);
        val.s = res;
    } else if (signature.isLong()) {
        // Value is a long
        CALL_STATIC_FUNCTION(CallStaticLongMethodA);
        val.j = res;
    } else if (signature.isFloat()) {
        // Value is a float
        CALL_STATIC_FUNCTION(CallStaticFloatMethodA);
        val.f = res;
    } else if (signature.isDouble()) {
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

Napi::Value jarray_to_napi_value(jarray array, const java_type &signature, const Napi::Env &env,
                                 const jni::jni_wrapper &j_env) {
    if (array == nullptr) {
        return env.Null();
    }

    const jsize length = j_env->GetArrayLength(array);
    j_env.checkForError();
    Napi::Array res = Napi::Array::New(env, length);

    if (signature.isInt()) {
        // Value is an integer
        jint *elements = j_env->GetIntArrayElements((jintArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::Number::From(env, elements[i]));
        }

        j_env->ReleaseIntArrayElements((jintArray) array, elements, 0);
    } else if (signature.isBool()) {
        // Value is a boolean
        jboolean *elements = j_env->GetBooleanArrayElements((jbooleanArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::Boolean::New(env, elements[i]));
        }

        j_env->ReleaseBooleanArrayElements((jbooleanArray) array, elements, 0);
    } else if (signature.isByte()) {
        // Value is a byte
        jni::jobject_wrapper<jbyteArray> byteArray((jbyteArray) array, j_env.env, false);
        jbyte *elements = j_env->GetByteArrayElements(byteArray, nullptr);
        j_env.checkForError();

        const auto releaser = [byteArray](const Napi::Env &, jbyte *arr) {
            jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
            j_env->ReleaseByteArrayElements(byteArray, arr, JNI_ABORT);
            j_env.checkForError();
        };

        return Napi::Buffer<jbyte>::New(env, elements, static_cast<size_t>(length), releaser);
    } else if (signature.isChar()) {
        // Value is a char
        jchar *elements = j_env->GetCharArrayElements((jcharArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::String::New(env, std::u16string({static_cast<char16_t>(elements[i])})));
        }

        j_env->ReleaseCharArrayElements((jcharArray) array, elements, 0);
    } else if (signature.isShort()) {
        // Value is a short
        jshort *elements = j_env->GetShortArrayElements((jshortArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::Number::From(env, elements[i]));
        }

        j_env->ReleaseShortArrayElements((jshortArray) array, elements, 0);
    } else if (signature.isLong()) {
        // Value is a long
        jlong *elements = j_env->GetLongArrayElements((jlongArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::BigInt::New(env, int64_t(elements[i])));
        }

        j_env->ReleaseLongArrayElements((jlongArray) array, elements, 0);
    } else if (signature.isFloat()) {
        // Value is a float
        jfloat *elements = j_env->GetFloatArrayElements((jfloatArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::Number::From(env, elements[i]));
        }

        j_env->ReleaseFloatArrayElements((jfloatArray) array, elements, 0);
    } else if (signature.isDouble()) {
        // Value is a double
        jdouble *elements = j_env->GetDoubleArrayElements((jdoubleArray) array, nullptr);
        j_env.checkForError();
        for (uint32_t i = 0; i < length; i++) {
            res.Set(i, Napi::Number::From(env, elements[i]));
        }

        j_env->ReleaseDoubleArrayElements((jdoubleArray) array, elements, 0);
    } else if (signature.type == j_type::Array) {
        // Value is an array
        for (uint32_t i = 0; i < length; i++) {
            jobject element = j_env->GetObjectArrayElement((jobjectArray) array, (jsize) i);
            j_env.checkForError();
            res.Set(i, jarray_to_napi_value((jarray) element, *signature.inner, env, j_env));
            j_env->DeleteLocalRef(element);
        }
    } else {
        for (uint32_t i = 0; i < length; i++) {
            jobject element = j_env->GetObjectArrayElement((jobjectArray) array, (jsize) i);
            j_env.checkForError();
            res.Set(i, conversion_helper::jobject_to_value(env, jni::jobject_wrapper<jobject>(element, j_env.env),
                                                           signature));
        }
    }

    return res;
}

Napi::Value
conversion_helper::jvalue_to_napi_value(jvalue value, const java_type &signature, const Napi::Env &env) {
    if (signature.isVoid()) {
        // Method returns void
        return env.Undefined();
    } else if (signature.isInt()) {
        // Value is an integer
        return Napi::Number::From(env, value.i);
    } else if (signature.isBool()) {
        // Value is a boolean
        return Napi::Boolean::New(env, value.z);
    } else if (signature.isByte()) {
        // Value is a byte
        return Napi::Number::From(env, value.b);
    } else if (signature.isChar()) {
        // Value is a char
        return Napi::String::New(env, std::string({static_cast<char>(value.c)}));
    } else if (signature.isShort()) {
        // Value is a short
        return Napi::Number::From(env, value.s);
    } else if (signature.isLong()) {
        // Value is a long
        return Napi::BigInt::New(env, int64_t(value.j));
    } else if (signature.isFloat()) {
        // Value is a float
        return Napi::Number::From(env, value.f);
    } else if (signature.isDouble()) {
        // Value is a double
        return Napi::Number::From(env, value.d);
    } else if (signature.isArray()) {
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
        return jarray_to_napi_value((jarray) value.l, *signature.inner, env, j_env);
    } else {
        // Value is some kind of object
        jni::jni_wrapper j_env = node_classes::jvm_container::attachJvm();
        jni::jobject_wrapper<jobject> obj(value.l, j_env.env, false);

        j_env->DeleteGlobalRef(value.l);
        return conversion_helper::jobject_to_value(env, obj, signature);
    }
}

bool conversion_helper::is_integer(const Napi::Env &env, const Napi::Number &num) {
    return env.Global()
            .Get("Number").As<Napi::Object>()
            .Get("isInteger").As<Napi::Function>()
            .Call({num}).ToBoolean().Value();
}
