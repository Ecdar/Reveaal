#include "dbm/fed.h"
#include "wrapper.h"
#include "dbm/constraints.h"
#include "dbm/dbm.h"
#include "hash/compute.h"

extern "C"
{
    raw_t dbm_boundbool2raw_wrapper(int32_t bound, bool isStrict)
    {
        return (bound * 2) | (isStrict ^ 1);
    }

    void dbm_zero_wrapper(raw_t *dbm, cindex_t dim)
    {
        base_fill(dbm, dbm + (dim * dim), dbm_LE_ZERO);
    }

    int32_t dbm_raw2bound_wrapper(raw_t raw) { return (raw >> 1); }
    bool dbm_rawIsStrict_wrapper(raw_t raw) { return ((raw & 1) ^ dbm_WEAK); }
    bool dbm_satisfies_wrapper(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j,
                               raw_t constraint)
    {
        assert(dbm && dim && i < dim && j < dim && dim > 0);
        return !(dbm[i * dim + j] > constraint &&             /* tightening ? */
                 dbm_negRaw(constraint) >= dbm[j * dim + i]); /* too tight ? */
    }

    dbm::fed_t dbm_subtract1_exposed(const raw_t *arg1, const raw_t *arg2, cindex_t dim)
    {
        return dbm::fed_t::subtract(arg1, arg2, dim);
    }

    dbm::fed_t dbm_subtract2_exposed(const dbm::dbm_t &arg1, const raw_t *arg2)
    {
        return dbm::fed_t::subtract(arg1, arg2);
    }

    dbm::fed_t dbm_subtract3_exposed(const dbm::dbm_t &arg1, const dbm::dbm_t &arg2)
    {
        return dbm::fed_t::subtract(arg1, arg2);
    }

    int dbm_check_validity(const raw_t *dbm, cindex_t dim)
    {
        try
        {
            if (dbm_isValid(dbm, dim) == true)
            {
                return 1;
            }
            else
            {
                return 0;
            }
        }
        catch (...)
        {
            return 0;
        }
    }

    void dbm_vec_to_fed(raw_t *dbm[], cindex_t len, cindex_t dim, dbm::fed_t *fed_out)
    {
        dbm::fed_t fed = (*new dbm::fed_t(dim));

        for (int i = 0; i < len; i++)
        {
            fed.add(dbm[i], dim);
        }
        fed_out->add(fed);
    }

    int dbm_get_fed_size(dbm::fed_t *fed)
    {
        return fed->size();
    }
    int dbm_get_fed_size_2(dbm::fed_t fed)
    {
        return fed.size();
    }
    int dbm_get_fed_dim(dbm::fed_t *fed)
    {
        return fed->getDimension();
    }

    int dbm_get_dbm_dimension(dbm::fed_t *fed)
    {
        return (fed->getDimension() * fed->getDimension());
    }

    const raw_t *dbm_get_ith_element_in_fed(dbm::fed_t *fed, int element_num)
    {
        int counter = 0;
        for (auto i = fed->begin(); i != fed->end(); ++i)
        {
            if (counter++ == element_num)
            {
                const raw_t *x = i->const_dbm();
                return x;
            }
        }
    }

    void dbm_fed_to_vec(dbm::fed_t &fed, const raw_t *head)
    {
        size_t dimension = fed.getDimension();
        //std::vector<dbm_t> vec;
        dbm::fdbm_t *prev = NULL;
        int y = 0;

        //const raw_t *x = fed.begin()->const_dbm();
        //head = x;
        for (auto i = fed.begin(); i != fed.end(); ++i)
        {
            const raw_t *x = i->const_dbm();
            dbm::fdbm_t *next = dbm::fdbm_t::create(x, dimension, prev);
            prev = next;
            //
            //
            //
            //            y++;
        }
        //
        //
        //        head = prev;
    }

    dbm::fdbm_t *dbm_create_fdbm_t()
    {
        dbm::fdbm_t *f = NULL;
        return f;
    }

    //dbm::fed_t dbm_fed_minus_fed(raw_t * dbm1[], raw_t * dbm2[], cindex_t len1, cindex_t len2, cindex_t dim) {

    void dbm_fed_minus_fed(raw_t *dbm1[], raw_t *dbm2[], cindex_t len1, cindex_t len2, cindex_t dim, dbm::fed_t *fed_out)
    {

        dbm::fed_t fed1 = (*new dbm::fed_t(dim));
        for (int i = 0; i < len1; i++)
        {
            fed1.add(dbm1[i], dim);
        }

        dbm::fed_t fed2 = (*new dbm::fed_t(dim));
        for (int i = 0; i < len2; i++)
        {
            fed2.add(dbm2[i], dim);
        }

        fed1 -= fed2;

        fed_out->add(fed1);
    }

    raw_t dbm_get_value(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j)
    {
        return dbm[i * dim + j];
    }
}
