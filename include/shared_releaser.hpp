#ifndef MARKUSJX_SHARED_RELEASER_HPP
#define MARKUSJX_SHARED_RELEASER_HPP

#include <functional>

/**
 * A class for calling a function when destroyed.
 * Works almost like a std::shared_ptr just without the pointer part.
 * Based on: https://codereview.stackexchange.com/questions/166395/custom-stdshared-ptrt-implementation
 */
class shared_releaser {
public:
    /**
     * Create a shared_releaser without setting a function
     */
    inline shared_releaser(std::nullptr_t) noexcept: on_destroy(nullptr), use_count(new std::size_t(0)) {}

    /**
     * Create a shared_releaser
     *
     * @param on_destroy the function to call on destruction
     */
    inline explicit shared_releaser(std::function<void()> on_destroy) : on_destroy(std::move(on_destroy)),
                                                                        use_count(new std::size_t(1)) {}

    /**
     * Create a shared_releaser from another instance
     *
     * @param rhs the other instance to copy from
     */
    inline shared_releaser(const shared_releaser &rhs) : use_count(rhs.use_count), on_destroy(rhs.on_destroy) {
        ++*use_count;
    }

    /**
     * Move constructor
     *
     * @param rhs the object to move
     */
    inline shared_releaser(shared_releaser &&rhs) noexcept: use_count(rhs.use_count), on_destroy(rhs.on_destroy) {
        rhs.use_count = new std::size_t(0);
        rhs.on_destroy = nullptr;
    }

    /**
     * Copy assign operator
     *
     * @param rhs the object to copy from
     * @return this
     */
    inline shared_releaser &operator=(const shared_releaser &rhs) {
        shared_releaser rhs_copy(rhs);
        this->swap(rhs_copy);
        return *this;
    }

    /**
     * Move assign operator
     *
     * @param rhs the object to move
     * @return this
     */
    inline shared_releaser &operator=(shared_releaser &&rhs) noexcept {
        this->swap(rhs);
        return *this;
    }

    /**
     * Swap this with another shared_releaser instance
     *
     * @param rhs the other instance to swap with
     */
    inline void swap(shared_releaser &rhs) {
        std::swap(this->use_count, rhs.use_count);
        std::swap(this->on_destroy, rhs.on_destroy);
    }

    /**
     * Check if the function is set
     *
     * @return true, if the function is not nullptr
     */
    [[nodiscard]] inline operator bool() const noexcept {
        return on_destroy.operator bool();
    }

    /**
     * Reset this. Calls the release function if the use count == 0
     *
     * @param new_fn the new function to swap with or nullptr to just call the release function
     */
    inline void reset(const std::function<void()> &new_fn = nullptr) {
        this->destructor();
        if (new_fn) {
            this->use_count = new std::size_t(1);
        } else {
            this->use_count = new std::size_t(0);
        }
        this->on_destroy = new_fn;
    }

    /**
     * The shared_releaser destructor.
     * Calls the release function if the use count equals zero
     */
    inline ~shared_releaser() {
        this->destructor();
    }

private:
    std::size_t *use_count;
    std::function<void()> on_destroy;

    /**
     * The destruction function
     */
    inline void destructor() {
        if (on_destroy && --*use_count <= 0) {
            on_destroy();
            delete use_count;
        } else if (!on_destroy) {
            delete use_count;
        }

        use_count = nullptr;
        on_destroy = nullptr;
    }
};

#endif //MARKUSJX_SHARED_RELEASER_HPP