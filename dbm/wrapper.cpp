#include "fed.h"
#include "wrapper.h"
#include "constraints.h"
#include "dbm.h"
#include <iostream>
#include <fstream>
#include <vector>

extern "C" {
    int32_t dbm_bound2raw_exposed(raw_t raw, strictness_t strict) 
    {

        return dbm_bound2raw(raw, strict);
    }

    int32_t dbm_boundbool2raw_exposed(raw_t raw, bool isStrict) 
    {

        return dbm_boundbool2raw(raw, isStrict);
    }

    int32_t dbm_raw2bound_exposed(raw_t raw)
    {
        return dbm_raw2bound(raw);
    }

    void dbm_zero_exposed(raw_t *dbm, cindex_t dim)
    {
        dbm_zero(dbm,dim);
    }

    BOOL dbm_rawIsStrict_exposed(raw_t raw)
    {
        return dbm_rawIsStrict(raw);
    }

    raw_t dbm_addRawRaw_exposed(raw_t x, raw_t y)
    {
        return dbm_addRawRaw(x, y);
    }

    dbm::fed_t dbm_subtract1_exposed(const raw_t* arg1, const raw_t* arg2, cindex_t dim)
    {
        return dbm::fed_t::subtract(arg1, arg2, dim);
    }

    dbm::fed_t dbm_subtract2_exposed(const dbm::dbm_t& arg1, const raw_t* arg2)
    {
        return dbm::fed_t::subtract(arg1, arg2);
    }

    dbm::fed_t dbm_subtract3_exposed(const dbm::dbm_t& arg1, const dbm::dbm_t& arg2)
    {
        return dbm::fed_t::subtract(arg1, arg2);
    }

    BOOL dbm_satisfies_exposed(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j, raw_t constraint)
    {
        return dbm_satisfies(dbm, dim, i, j, constraint);
    }

    void dbm_vec_to_fed( raw_t * dbm[], cindex_t len, cindex_t dim, dbm::fed_t * fed_out)
    {
        std::ofstream myfile;
        myfile.open ("vec_to_fed.txt");

        dbm::fed_t fed = (*new dbm::fed_t(dim));

        for (int i = 0; i < len; i++) {
            fed.add(dbm[i], dim);
        }
        fed_out->add(fed) ;

        myfile.close();
    }



    void dbm_fed_to_vec( dbm::fed_t &fed, const raw_t *head)
    {
        size_t dimension = fed.getDimension();
        std::ofstream myfile;
        myfile.open ("fed_to_vec_test.txt");
        //std::vector<dbm_t> vec;
        dbm::fdbm_t * prev = NULL;
        int y = 0;
        myfile<<fed.isEmpty();

        const raw_t *x = fed.begin()->const_dbm();
        head = x;
        for (auto i = fed.begin(); i != fed.end(); ++i) {
            //myfile << prev <<"lvl2\n";
            const raw_t *x = i->const_dbm();
            dbm::fdbm_t * next = dbm::fdbm_t::create(x, dimension, prev);
            prev = next;
//
//
//
//            y++;
        }
//
//
        myfile.close();
//        head = prev;

    }

    dbm::fdbm_t* dbm_create_fdbm_t() {
        dbm::fdbm_t * f = NULL;
        return f;
    }

    void dbm_fed_minus_fed(dbm::fed_t &fed1, dbm::fed_t &fed2, dbm::fed_t * fed_out) {
        std::ofstream myfile;
        myfile.open ("test.txt");
        myfile<< fed1.getDimension()<<"\n";
        myfile<< fed2.getDimension()<<"\n";

        dbm::fed_t res = fed1 - fed2;
        dbm::fed_t * res_ptr = &res;
        fed_out->setDimension(res.getDimension());
        myfile<<"pointer dim: "<<res_ptr->getDimension()<<"\n";
        myfile<<"pointer len: "<<res_ptr->size()<<"\n";
        //myfile<< res.getDimension()<<"\n";

        fed_out=res_ptr;
        myfile<<"fed_out dim: "<<fed_out->getDimension()<<"\n";
        myfile<<"fed_out len: "<<fed_out->size()<<"\n";
        myfile.close();
    }

    raw_t dbm_get_value(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j) {
        return dbm[i*dim + j];
    }
}

