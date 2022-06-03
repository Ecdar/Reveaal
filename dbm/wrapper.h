#ifndef WRAPPER_H
#define WRAPPER_H
#include "dbm/dbm.h"

extern "C"
{
    raw_t dbm_boundbool2raw_wrapper(int32_t bound, bool isStrict);
    int32_t dbm_raw2bound_wrapper(raw_t raw);
    bool dbm_rawIsStrict_wrapper(raw_t raw);
    bool dbm_satisfies_wrapper(const dbm::dbm_t *dbm, cindex_t i, cindex_t j,
                               raw_t constraint);

    bool dbm_check_validity(const dbm::dbm_t *dbm);

    raw_t dbm_get_value(const dbm::dbm_t *dbm, cindex_t i, cindex_t j);

    void fed_get_dbm_vec(const dbm::fed_t *fed, raw_t *vec, size_t vec_len);

    void fed_subtraction(dbm::fed_t &fed1, const dbm::fed_t &fed2);

    void fed_intersection(dbm::fed_t &fed1, const dbm::fed_t &fed2);

    bool fed_intersects(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    bool fed_is_valid(const dbm::fed_t &fed);

    bool fed_is_empty(const dbm::fed_t &fed);

    void fed_up(dbm::fed_t &fed);

    void fed_down(dbm::fed_t &fed);

    void fed_init(dbm::fed_t &fed);

    void fed_zero(dbm::fed_t &fed);

    void fed_predt(dbm::fed_t &good, const dbm::fed_t &bad);

    // maybe call with
    //  let mut ptr = ::std::mem::MaybeUninit::uninit();
    //  func(ptr.as_mut_ptr());
    //  ptr.assume_init()

    bool fed_constrain(dbm::fed_t &fed, cindex_t i, cindex_t j, int32_t b, bool isStrict);

    /// Update method where x & y are clocks, v an integer value.
    ///     x := y + v -> update
    void fed_update(dbm::fed_t &fed, cindex_t x, cindex_t y, int32_t v);

    void fed_free_clock(dbm::fed_t &fed, cindex_t x);

    bool fed_subset_eq(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    relation_t fed_relation(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    relation_t fed_exact_relation(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    bool fed_eq(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    bool fed_exact_eq(const dbm::fed_t &fed1, const dbm::fed_t &fed2);

    void fed_reduce(dbm::fed_t &fed);

    void fed_expensive_reduce(dbm::fed_t &fed);

    // fed.isUnbounded()
    bool fed_can_delay_indef(const dbm::fed_t &fed);

    void fed_extrapolate_max_bounds(dbm::fed_t &fed, const int32_t *max);

    void fed_diagonal_extrapolate_max_bounds(dbm::fed_t &fed, const int32_t *max);

    void fed_add_fed(dbm::fed_t &fed, const dbm::fed_t &other);

    void fed_invert(dbm::fed_t &fed);

    size_t fed_size(const dbm::fed_t &fed);

    void fed_destruct(dbm::fed_t &fed);

    bool fed_is_mutable(dbm::fed_t &fed);

    void fed_new(dbm::fed_t &fed, cindex_t dim);

    cindex_t fed_dimension(const dbm::fed_t &fed);

    // void fed_crash(cindex_t dim);
}
//
#endif // WRAPPER_H
