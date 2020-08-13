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
        myfile<<"Fed size:"<<fed_out->size();
        myfile.close();
    }

    int dbm_get_fed_size(dbm::fed_t * fed){
        return fed->size();
    }
    int dbm_get_fed_size_2(dbm::fed_t fed){
    return fed.size();
    }

    int dbm_get_dbm_dimension(dbm::fed_t * fed){
        return (fed->getDimension() * fed->getDimension());
    }


    const raw_t * dbm_get_ith_element_in_fed(dbm::fed_t * fed, int element_num){
        int counter = 0;
        for (auto i = fed->begin(); i != fed->end(); ++i) {
            if (counter == element_num) {
                const raw_t *x = i->const_dbm();
                return x;
            }
        }
    }


    void dbm_fed_to_vec( dbm::fed_t &fed, const raw_t *head)
    {
        size_t dimension = fed.getDimension();
        std::ofstream myfile;
        myfile.open ("fed_to_vec_test.txt");
        //std::vector<dbm_t> vec;
        dbm::fdbm_t * prev = NULL;
        int y = 0;
        myfile<<fed.isEmpty()<<"\n";
        myfile<<"fed size: "<<fed.size();

        //const raw_t *x = fed.begin()->const_dbm();
        //head = x;
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

    dbm::fed_t dbm_fed_minus_fed(dbm::fed_t &fed1, dbm::fed_t &fed2) {
        std::ofstream myfile;
        myfile.open ("test.txt");

        int dbm[9];
        size_t dim = 3;
        dbm_init(dbm,dim);

        dbm::fed_t fed_test_1 = (*new dbm::fed_t(dim));
        /*       fed_test_1.add(dbm, dim);
               dbm::fed_t fed_test_2 = (*new dbm::fed_t(dim));

               dbm::fed_t res_test = fed_test_1 -fed_test_2;
               dbm::fed_t * res_ptr = &res_test ;


               myfile<< "size of TEST::: " << res_test.size()<<"\n";
       */
        // dbm_init(dbm.as_mut_ptr(), dimension);
        
        // myfile<< "size leftside: " <<fed1->size()<<"\n";
        // myfile<< "size rightside: " <<fed2->size()<<"\n";
        /*for (auto i = fed1.begin(); i != fed1.end(); ++i) {
            const int dbm[fed2.getDimension() * fed2.getDimension()];
            const raw_t *x = i->const_dbm();
            &dbm = x;
            fed_test_1.add(x, fed1.getDimension());
        }

        dbm::fed_t fed_test_2 = (*new dbm::fed_t(dim));
        for (auto i = fed2.begin(); i != fed2.end(); ++i) {
            const int dbm[fed2.getDimension() * fed2.getDimension()];
            const raw_t *x = i->const_dbm();
            &dbm = x;
            fed_test_2.add(x, fed2.getDimension());
        }*/
        //dbm::fed_t res = fed1 - fed2;
        //dbm::fed_t * res_ptr = &res;
    
        // myfile<<"pointer dim: "<<res_ptr->getDimension()<<"\n";
        // myfile<<"pointer len: "<<res_ptr->size()<<"\n";

        //fed_out=res_ptr;
        //myfile<<"fed_out dim: "<<fed_out->getDimension()<<"\n";
        //myfile<<"fed_out len: "<<fed_out->size()<<"\n";
        size_t dim1 = fed1.getDimension();
        myfile<<"fed dim is: "<<dim1<<"\n";
        myfile<<"fed 1 len before is: "<<fed1.size()<<"\n";
        myfile<<"fed test len is: " << fed_test_1.size()<<"\n";

        fed1 -= fed_test_1;
        myfile<<"fed 2 len after is: "<<fed1.size();


        //dbm::fed_t res = fed1 - fed2;
        //dbm::fed_t * res_ptr = &res;
        myfile.close();

        return fed1;
        //dbm::fed_t * ptr = &res_test;
        
        //fed_out = ptr;
        //myfile<<"fed_out size:: " << fed_out->size()<<"\n";
        
    }

    raw_t dbm_get_value(const raw_t *dbm, cindex_t dim, cindex_t i, cindex_t j) {
        return dbm[i*dim + j];
    }
}

