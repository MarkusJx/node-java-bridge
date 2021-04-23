#ifndef NODE_JAVA_BRIDGE_DEFINITIONS_HPP
#define NODE_JAVA_BRIDGE_DEFINITIONS_HPP

#if defined(WIN32) || defined(_WIN32) || defined(__WIN32) && !defined(__CYGWIN__)
#   define JAVA_WINDOWS
#elif defined(__LINUX__) || defined(__APPLE__) || defined (__CYGWIN__) || defined(__linux__) || defined(__FreeBSD__) || \
        defined(unix) || defined(__unix) || defined(__unix__)
#   undef JAVA_WINDOWS
#endif

#endif //NODE_JAVA_BRIDGE_DEFINITIONS_HPP
