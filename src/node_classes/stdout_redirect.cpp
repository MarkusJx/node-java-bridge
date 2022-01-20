#include <stdexcept>
#include <napi_tools.hpp>

#include "node_classes/stdout_redirect.hpp"
#include "jvm_lib/io_github_markusjx_bridge_StdoutRedirect_CallbackOutputStream.h"
#include "node_classes/jvm_container.hpp"

using namespace node_classes;

using callback = napi_tools::callbacks::callback<void(std::string)>;
using callback_ptr = std::shared_ptr<callback>;

callback_ptr stdout_callback;
callback_ptr stderr_callback;
static std::mutex mtx;
static jni::jobject_wrapper<jobject> classInstance;
static jclass clazz = nullptr;

JAVA_UNUSED JNIEXPORT void
JNICALL Java_io_github_markusjx_bridge_StdoutRedirect_00024CallbackOutputStream_writeLine(JNIEnv *env, jobject,
                                                                                          jstring to_write,
                                                                                          jboolean is_stdout) {
    try {
        callback_ptr out, err;
        {
            std::unique_lock lock(mtx);
            out = stdout_callback;
            err = stderr_callback;
        }

        const char *chars = env->GetStringUTFChars(to_write, nullptr);
        const std::string data(chars);
        env->ReleaseStringUTFChars(to_write, chars);

        if (is_stdout && out) {
            out->call(data);
        } else if (!is_stdout && err) {
            err->call(data);
        }
    } catch (const std::exception &e) {
        env->ThrowNew(env->FindClass("java/lang/Exception"), e.what());
    }
}

void reset(const Napi::CallbackInfo &info) {
    TRY
        std::unique_lock lock(mtx);
        stdout_callback.reset();
        stderr_callback.reset();
        if (!classInstance.isNull()) {
            jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
            static jmethodID rst = jvm->GetMethodID(clazz, "reset", "()V");
            jvm.checkForError();

            jvm->CallVoidMethod(classInstance, rst);
            jvm.checkForError();
        }

        classInstance.reset();
    CATCH_EXCEPTIONS
}

void setCallbacks(const Napi::CallbackInfo &info) {
    static const auto mask = napi_tools::function | napi_tools::null | napi_tools::undefined;
    CHECK_ARGS(mask, mask);

    reset(info);
    if (!info[0].IsFunction() && !info[1].IsFunction()) {
        return;
    }

    TRY
        jni::jni_wrapper jvm = node_classes::jvm_container::attachJvm();
        if (!clazz) clazz = jvm.getJClass("io.github.markusjx.bridge.StdoutRedirect");
        jvm.checkForError();

        static jmethodID ctor = jvm->GetMethodID(clazz, "<init>", "(ZZ)V");
        jvm.checkForError();

        std::unique_lock lock(mtx);
        classInstance = jni::jobject_wrapper(jvm->NewObject(clazz, ctor, info[0].IsFunction(), info[1].IsFunction()),
                                             jvm);

        if (info[0].IsFunction()) {
            stdout_callback = std::make_shared<callback>(info.Env(), info[0].As<Napi::Function>());
        }

        if (info[1].IsFunction()) {
            stderr_callback = std::make_shared<callback>(info.Env(), info[1].As<Napi::Function>());
        }
    CATCH_EXCEPTIONS
}

void stdout_redirect::init(Napi::Env env, Napi::Object &exports) {
    Napi::Object obj = Napi::Object::New(env);
    EXPORT_FUNCTION(obj, env, reset);
    EXPORT_FUNCTION(obj, env, setCallbacks);

    exports.Set("stdout_redirect", obj);
}
