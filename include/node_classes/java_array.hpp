#ifndef NODE_JAVA_BRIDGE_JAVA_ARRAY_HPP
#define NODE_JAVA_BRIDGE_JAVA_ARRAY_HPP

#include <napi.h>

namespace node_classes {
    class java_array : public Napi::ObjectWrap<java_array> {
    public:
        static void init(Napi::Env env, Napi::Object &exports);


    private:
        static Napi::FunctionReference *constructor;
    };
}

#endif //NODE_JAVA_BRIDGE_JAVA_ARRAY_HPP
