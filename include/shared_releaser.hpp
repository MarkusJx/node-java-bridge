#ifndef MARKUSJX_SHARED_RELEASER_HPP
#define MARKUSJX_SHARED_RELEASER_HPP

#include <functional>
#include <mutex>

/**
 * A class for calling a function when destroyed.
 * Works almost like a std::shared_ptr just without the pointer part.
 * Based on: https://codereview.stackexchange.com/questions/166395/custom-stdshared-ptrt-implementation
 * Was originally used and created by and for my autobet project.
 *
 * @author MarkusJx
 */
class shared_releaser {
public:
    /**
     * Create a shared_releaser without setting a function
     */
    shared_releaser(std::nullptr_t) noexcept: on_destroy(nullptr), use_count(new std::size_t(0)),
                                              mtx(new std::mutex()) {}

    /**
     * Create a shared_releaser
     *
     * @param on_destroy the function to call on destruction
     */
    explicit shared_releaser(std::function<void()> on_destroy) : on_destroy(std::move(on_destroy)),
                                                                 use_count(new std::size_t(1)),
                                                                 mtx(new std::mutex()) {}

    /**
     * Create a shared_releaser from another instance
     *
     * @param rhs the other instance to copy from
     */
    shared_releaser(const shared_releaser &rhs) : use_count(rhs.use_count), on_destroy(rhs.on_destroy), mtx(rhs.mtx) {
        if (mtx) std::unique_lock<std::mutex> lock(*mtx);
        ++*use_count;
    }

    /**
     * Move constructor
     *
     * @param rhs the object to move
     */
    shared_releaser(shared_releaser &&rhs) noexcept: use_count(rhs.use_count), on_destroy(rhs.on_destroy),
                                                     mtx(rhs.mtx) {
        rhs.use_count = new std::size_t(0);
        rhs.on_destroy = nullptr;
        rhs.mtx = nullptr;
    }

    /**
     * Copy assign operator
     *
     * @param rhs the object to copy from
     * @return this
     */
    shared_releaser &operator=(const shared_releaser &rhs) {
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
    shared_releaser &operator=(shared_releaser &&rhs) noexcept {
        this->swap(rhs);
        return *this;
    }

    /**
     * Swap this with another shared_releaser instance
     *
     * @param rhs the other instance to swap with
     */
    void swap(shared_releaser &rhs) {
        std::swap(this->use_count, rhs.use_count);
        std::swap(this->on_destroy, rhs.on_destroy);
        std::swap(this->mtx, rhs.mtx);
    }

    /**
     * Assign another shared_releaser to this
     *
     * @param rhs the releaser to assign to this
     */
    void assign(const shared_releaser &rhs) {
        this->destructor();
        this->use_count = rhs.use_count;
        this->on_destroy = rhs.on_destroy;
        this->mtx = rhs.mtx;

        if (mtx) std::unique_lock<std::mutex> lock(*mtx);
        ++*use_count;
    }

    /**
     * Check if the function is set
     *
     * @return true, if the function is not nullptr
     */
    [[nodiscard]] operator bool() const noexcept {
        return on_destroy.operator bool();
    }

    /**
     * Reset this. Calls the release function if the use count == 0
     *
     * @param new_fn the new function to swap with or nullptr to just call the release function
     */
    void reset(const std::function<void()> &new_fn = nullptr) {
        this->destructor();
        if (new_fn) {
            this->use_count = new std::size_t(1);
        } else {
            this->use_count = new std::size_t(0);
        }
        this->on_destroy = new_fn;
        this->mtx = new std::mutex();
    }

    /**
     * The shared_releaser destructor.
     * Calls the release function if the use count equals zero
     */
    ~shared_releaser() {
        this->destructor();
    }

private:
    std::size_t *use_count;
    std::function<void()> on_destroy;
    std::mutex *mtx;

    /**
     * The destruction function
     */
    void destructor() {
        if (mtx)
            mtx->lock();

        if (on_destroy && --*use_count <= 0) {
            if (mtx)
                mtx->unlock();

            on_destroy();
            delete mtx;
            delete use_count;
        } else if (!on_destroy) {
            if (mtx)
                mtx->unlock();

            delete mtx;
            delete use_count;
        } else if (mtx) {
            mtx->unlock();
        }

        use_count = nullptr;
        on_destroy = nullptr;
        mtx = nullptr;
    }
};

#endif //MARKUSJX_SHARED_RELEASER_HPP