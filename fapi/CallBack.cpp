/**
 * @file CallBack.cpp
 *
 * @date 23 March 2005
 *
 * @brief Stores Client data relating to function call back registration by a 
 *        FAPI Sim client.
 * 
 * An instance of this class is created to store userContext, callback function 
 * pointer, callback correlator and callback data per client registered 
 * callback.The object also invokes the callback function pointer with relevant 
 * parameters.
 *
 *
 * -- Intel Copyright Notice --
 *
 * @par
 * INTEL CONFIDENTIAL
 *
 * @par
 * Copyright 2005 Intel Corporation All Rights Reserved
 *
 * @par
 * The source code contained or described herein and all documents
 * related to the source code ("Material") are owned by Intel Corporation
 * or its suppliers or licensors.  Title to the Material remains with
 * Intel Corporation or its suppliers and licensors.  The Material
 * contains trade secrets and proprietary and confidential information of
 * Intel or its suppliers and licensors.  The Material is protected by
 * worldwide copyright and trade secret laws and treaty provisions. No
 * part of the Material may be used, copied, reproduced, modified,
 * published, uploaded, posted, transmitted, distributed, or disclosed in
 * any way without Intel's prior express written permission.
 *
 * @par
 * No license under any patent, copyright, trade secret or other
 * intellectual property right is granted to or conferred upon you by
 * disclosure or delivery of the Materials, either expressly, by
 * implication, inducement, estoppel or otherwise.  Any license under
 * such intellectual property rights must be express and approved by
 * Intel in writing.
 *
 * @par
 * For further details, please see the file README.TXT distributed with
 * this software.
 * -- End Intel Copyright Notice –
 */

/*
 * User defined include files required.
 */
#include "CallBack.h"
#include <iostream>
#include "TraceMacro.h"

CallBack::CallBack(NPF_userContext_t userContext,
                   NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc)
                   :m_cbCorrelator(0), m_context(userContext), m_function(callbackFunc) 
{
}

CallBack::~CallBack()
{
}

/**
 * Function Defintion: fire()
 */
void CallBack::Fire()
{
    APISimTrace(3,"Trace Level 3: CallBack::Fire()\n");
    // Invoke client function pointer. 
    m_function(m_context, m_cbCorrelator, m_data); 
    delete [] m_data.resp; 
    delete this;

}
