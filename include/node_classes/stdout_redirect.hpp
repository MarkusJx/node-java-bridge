#ifndef NODE_JAVA_BRIDGE_STDOUT_REDIRECT_HPP
#define NODE_JAVA_BRIDGE_STDOUT_REDIRECT_HPP

#include <napi.h>

namespace node_classes {
    /**
     * A namespace for redirecting the stdout from java
     */
    namespace stdout_redirect {
        /**
         * Initialize the object and export it
         */
        void init(Napi::Env env, Napi::Object &exports);
    }
}

#endif //NODE_JAVA_BRIDGE_STDOUT_REDIRECT_HPP
