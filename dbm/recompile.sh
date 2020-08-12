rm objectFiles/wrapper.o
g++ -c wrapper.cpp objectFiles/*.o -o objectFiles/wrapper.o
ar rvs libdbmfull.a objectFiles/*.o