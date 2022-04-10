#include "dbm/fed.h"
#include "wrapper.h"
#include "dbm/constraints.h"
#include "dbm/dbm.h"
#include "hash/compute.h"

raw_t dbm_boundbool2raw_wrapper(int32_t bound, bool isStrict)
{
    return (bound * 2) | (isStrict ^ 1);
}

int32_t dbm_raw2bound_wrapper(raw_t raw) { return (raw >> 1); }
bool dbm_rawIsStrict_wrapper(raw_t raw) { return ((raw & 1) ^ dbm_WEAK); }
bool dbm_satisfies_wrapper(const dbm::dbm_t &dbm, cindex_t i, cindex_t j,
                           raw_t constraint)
{
    cindex_t dim = dbm.getDimension();
    assert(dim > 0 && i < dim && j < dim);
    return !(dbm(i, j) > constraint &&             /* tightening ? */
             dbm_negRaw(constraint) >= dbm(j, i)); /* too tight ? */
}

bool dbm_check_validity(const dbm::dbm_t *dbm)
{
    try
    {
        if (dbm_isValid(dbm->const_dbm(), dbm->getDimension()) == true)
        {
            return true;
        }
        else
        {
            return false;
        }
    }
    catch (...)
    {
        return false;
    }
}

void fed_get_dbm_vec(const dbm::fed_t *fed, raw_t *vec, size_t vec_len)
{
    cindex_t dim = fed->getDimension();
    assert(vec_len == fed->size() * dim * dim);

    for (auto dbm = fed->begin(); dbm != fed->end(); ++dbm)
    {
        for (cindex_t i = 0; i < dim; ++i)
        {
            for (cindex_t j = 0; j < dim; ++j, ++vec)
            {
                *vec = (*dbm)(i, j);
            }
        }
    }
}

raw_t dbm_get_value(const dbm::dbm_t &dbm, cindex_t i, cindex_t j)
{
    return dbm(i, j);
}

void fed_subtraction(dbm::fed_t &fed1, const dbm::fed_t &fed2)
{
    fed1 -= fed2;
}

void fed_intersection(dbm::fed_t &fed1, const dbm::fed_t &fed2)
{
    fed1 &= fed2;
}

bool fed_is_valid(const dbm::fed_t &fed)
{
    return !fed.isEmpty();
}

bool fed_is_empty(const dbm::fed_t &fed)
{
    return fed.isEmpty();
}

void fed_up(dbm::fed_t &fed)
{
    fed.up();
}

void fed_down(dbm::fed_t &fed)
{
    fed.down();
}

void fed_init(dbm::fed_t &fed)
{
    fed.setInit();
}

void fed_zero(dbm::fed_t &fed)
{
    fed.setZero();
}

bool fed_intersects(const dbm::fed_t &fed1, const dbm::fed_t &fed2)
{
    return fed1.intersects(fed2);
}

bool fed_constrain(dbm::fed_t &fed, cindex_t i, cindex_t j, int32_t b, bool isStrict)
{
    return fed.constrain(i, j, b, isStrict);
}

void fed_update(dbm::fed_t &fed, cindex_t x, cindex_t y, int32_t v)
{
    fed.update(x, y, v);
}

void fed_free_clock(dbm::fed_t &fed, cindex_t x)
{
    fed.freeClock(x);
}

bool fed_subset_eq(const dbm::fed_t &fed1, const dbm::fed_t &fed2)
{
    return fed1 <= fed2;
}

relation_t fed_relation(const dbm::fed_t &fed1, const dbm::fed_t &fed2, bool exact)
{
    if (exact)
    {
        return fed1.exactRelation(fed2);
    }
    else
    {
        return fed1.relation(fed2);
    }
}

bool fed_eq(const dbm::fed_t &fed1, const dbm::fed_t &fed2, bool exact)
{
    if (exact)
    {
        return fed1.eq(fed2);
    }
    else
    {
        return fed1 == fed2;
    }
}

void fed_reduce(dbm::fed_t &fed, bool expensive)
{
    if (expensive)
    {
        fed.expensiveReduce();
    }
    else
    {
        fed.reduce();
    }
}

bool fed_can_delay_indef(const dbm::fed_t &fed)
{
    return fed.isUnbounded();
}

void fed_extrapolate_max_bounds(dbm::fed_t &fed, const int32_t *max)
{
    fed.extrapolateMaxBounds(max);
}

void fed_add_fed(dbm::fed_t &fed, const dbm::fed_t &other)
{
    fed.add(other);
}

void fed_invert(dbm::fed_t &fed)
{
    fed = !fed;
}

size_t fed_size(const dbm::fed_t &fed)
{
    return fed.size();
}

bool fed_is_mutable(dbm::fed_t &fed)
{
    return fed.ifed()->isMutable();
}

void fed_new(dbm::fed_t &fed, cindex_t dim)
{
    fed = dbm::fed_t(dim);
}

void fed_destruct(dbm::fed_t &fed)
{
    fed.nil();
    fed.~fed_t();
}

cindex_t fed_dimension(const dbm::fed_t &fed)
{
    return fed.getDimension();
}

/*
void fed_crash(cindex_t dim)
{
    dbm::fed_t fed(dim);
    fed.setZero();
    fed.nil();
}*/