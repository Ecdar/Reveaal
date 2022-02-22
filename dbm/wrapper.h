#ifndef WRAPPER_H
#define WRAPPER_H
#include "dbm/dbm.h"

extern "C"
{
    /// Subtract DBM arg1 - DBM arg2 wrapper functions.
    dbm::fed_t dbm_subtract1_exposed(const raw_t *arg1, const raw_t *arg2, cindex_t dim);
    dbm::fed_t dbm_subtract2_exposed(const dbm::dbm_t &arg1, const raw_t *arg2);
    dbm::fed_t dbm_subtract3_exposed(const dbm::dbm_t &arg1, const dbm::dbm_t &arg2);

    void dbm_fed_minus_fed(raw_t *dbm1[], raw_t *dbm2[], cindex_t len1, cindex_t len2, cindex_t dim, dbm::fed_t *fed_out);

    raw_t dbm_get_value(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j);

    /** Create a federation from vector of dbms
     * @param dbm: vector of DBMs
     * @param dim: dimension
     * @return Federation from DBMs
     * @post Federation
     */
    void dbm_vec_to_fed(raw_t *dbm[], cindex_t len, cindex_t dim, dbm::fed_t *fed_out);
    int dbm_get_fed_size_2(dbm::fed_t fed);

    int dbm_get_fed_dim(dbm::fed_t *fed);

    int dbm_get_fed_size(dbm::fed_t *fed);

    int dbm_get_dbm_dimension(dbm::fed_t *fed);

    const raw_t *dbm_get_ith_element_in_fed(dbm::fed_t *fed, int element_num);
    int dbm_get_fed_size_2(dbm::fed_t fed);

    int dbm_check_validity(const raw_t *dbm, cindex_t dim);

    raw_t dbm_boundbool2raw_wrapper(int32_t bound, bool isStrict);
    void dbm_zero_wrapper(raw_t *dbm, cindex_t dim);
    int32_t dbm_raw2bound_wrapper(raw_t raw);
    bool dbm_rawIsStrict_wrapper(raw_t raw);
    bool dbm_satisfies_wrapper(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j,
                               raw_t constraint);
}

#endif // WRAPPER_H