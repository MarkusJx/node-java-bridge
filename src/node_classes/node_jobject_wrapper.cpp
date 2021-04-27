#include "node_classes/node_jobject_wrapper.hpp"

using namespace node_classes;

void node_jobject_wrapper::init(Napi::Env env, Napi::Object &exports) {
    Napi::Function func = DefineClass(env, "node_jobject_wrapper", {});

    constructor = new Napi::FunctionReference();
    *constructor = Napi::Persistent(func);
    exports.Set("node_jobject_wrapper", func);

    env.SetInstanceData<Napi::FunctionReference>(constructor);
}

Napi::Object node_jobject_wrapper::createInstance() {
    if (!constructor) {
        throw std::runtime_error("The constructor pointer was null");
    }

    return constructor->New({});
}

bool node_jobject_wrapper::instanceOf(const Napi::Object &obj) {
    if (constructor) {
        return obj.InstanceOf(constructor->Value());
    } else {
        throw std::runtime_error("The constructor pointer was null");
    }
}

node_jobject_wrapper::node_jobject_wrapper(const Napi::CallbackInfo &info) : ObjectWrap(info), object() {}

void node_jobject_wrapper::setData(const jni::jobject_wrapper<jobject> &obj) {
    object = obj;
}

jni::jobject_wrapper<jobject> node_jobject_wrapper::getObject() const {
    return object;
}

Napi::FunctionReference *node_jobject_wrapper::constructor = nullptr;