#include <napi_tools.hpp>

#include "util/util.hpp"
#include "node_classes/jvm_container.hpp"

using namespace node_classes;

void jvm_container::createInstance(const std::string &lib_path, jint version) {
    instance = std::make_unique<jvm_container>(lib_path, version);
}

void jvm_container::destroyInstance() {
    if (instance) {
        instance.reset();
    }
}

jvm_container::jvm_container(const std::string &lib_path, jint version) : root_jvm(
        jni::jvm_wrapper::create_jvm_wrapper(lib_path, version)) {}

jni::jvm_wrapper &jvm_container::getJvm() {
    if (!instance) {
        throw std::runtime_error("The jvm_container was not initialized");
    } else {
        return instance->root_jvm;
    }
}

jni::jni_wrapper jvm_container::attachJvm() {
    return getJvm().attachEnv();
}

jvm_container::~jvm_container() {
    // Force reset the jvm in the destructor
    // as any later calls to destroy the
    // jvm will cause the jvm to crash.
    // I don't know why this happens, but I can
    // prevent it using this thing.
    // Those crashes only happen occasionally,
    // if not many operations are executed.
    root_jvm.env.forceReset();
}

std::unique_ptr<jvm_container> jvm_container::instance = nullptr;
