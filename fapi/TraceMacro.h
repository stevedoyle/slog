#ifndef _TRACEMACRO_H_
#define _TRACEMACRO_H_
#include "iostream.h"


#define APISimTrace(level, format, args...)(level > 4) ? :  printf(format, ##args)



#endif //_TRACEMACRO_H_
