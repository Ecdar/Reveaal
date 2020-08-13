#include "fed.h"
#include "constraints.h"
#include "dbm.h"

extern "C" {

    /** Encoding of bound into (strict) less or less equal.
     * @param bound,strict: the bound to encode with the strictness.
     * @return encoded constraint ("raw").
     */
    int32_t dbm_bound2raw_exposed(raw_t raw, strictness_t strict);

    /** Encoding of bound into (strict) less or less equal.
     * @param bound,isStrict: the bound to encode with a flag
     * telling if the bound is strict or not.
     * if isStrict is TRUE then dbm_STRICT is taken,
     * otherwise dbm_WEAK.
     * @return encoded constraint ("raw").
     */
    int32_t dbm_boundbool2raw_exposed(raw_t raw, bool isStrict);

    /** Decoding of raw representation: bound.
     * @param raw: encoded constraint (bound + strictness).
     * @return the decoded bound value.
     */
    int32_t dbm_raw2bound_exposed(raw_t raw);

    /** Set the DBM so that it contains only 0.
     * @param dbm: DBM to set to 0
     * @param dim: dimension
     * @return zeroed DBM
     * @post DBM is closed
     */
    void dbm_zero_exposed(raw_t *dbm, cindex_t dim);

    /** Tests of strictness.
     * @param raw: encoded constraint (bound + strictness).
     * @return TRUE if the constraint is strict.
     * dbm_rawIsStrict(x) == !dbm_rawIsEq(x)
     */
    BOOL dbm_rawIsStrict_exposed(raw_t raw);

    /** Constraint addition on raw values : + constraints - excess bit.
     * @param x,y: encoded constraints to add.
     * @return encoded constraint x+y.
     */
    raw_t dbm_addRawRaw_exposed(raw_t x, raw_t y);

    /// Subtract DBM arg1 - DBM arg2 wrapper functions.
    dbm::fed_t dbm_subtract1_exposed(const raw_t* arg1, const raw_t* arg2, cindex_t dim);
    dbm::fed_t dbm_subtract2_exposed(const dbm::dbm_t& arg1, const raw_t* arg2);
    dbm::fed_t dbm_subtract3_exposed(const dbm::dbm_t& arg1, const dbm::dbm_t& arg2);

    /** Satisfy operation.
     * Check if a DBM satisfies a constraint. The DBM is not modified.
     * WARNING: using this for conjunction of constraints is incorrect
     * because the DBM is not modified.
     * @param dbm: DBM.
     * @param dim: dimension.
     * @param i,j: indices of clocks for the clock constraint.
     * @param constraint: the encoded constraint.
     * @pre
     * - DBM is closed and non empty.
     * - dim > 0
     * - i != j (don't touch the diagonal)
     * - i < dim, j < dim
     * @return TRUE if the DBM satisfies the constraint.
     */
    BOOL dbm_satisfies_exposed(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j, raw_t constraint);


    void dbm_fed_minus_fed(const dbm::fed_t &fed1, const dbm::fed_t &fed2, dbm::fed_t * fed_out);


    raw_t dbm_get_value(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j);

    /** Create a federation from vector of dbms
     * @param dbm: vector of DBMs
     * @param dim: dimension
     * @return Federation from DBMs
     * @post Federation
     */
    void dbm_vec_to_fed( raw_t * dbm[], cindex_t len, cindex_t dim, dbm::fed_t * fed_out);



    int dbm_get_fed_size(dbm::fed_t * fed);

    int dbm_get_dbm_dimension(dbm::fed_t * fed);

    const raw_t * dbm_get_ith_element_in_fed(dbm::fed_t * fed, int element_num);


}