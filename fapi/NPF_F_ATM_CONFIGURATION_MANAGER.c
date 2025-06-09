/**
 * @file NPF_F_ATM_CONFIGURATION_MANAGER.C
 *
 * @date 22 March 2005
 *
 * @brief Implementation of NPF ATM Configuration Manager implementation agreement. 
 *
 * This file provides the implementation for all operations defined
 * in the NPF ATM Configuration Manager implementation agreement. The operations
 * within represent part of the external facade of the FAPI simulator.
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
#include "npf.h"
#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"
#include "CallBackManager.h"
#include "TableManager.h"
#include "CallBackHandler.h"
#include "TraceMacro.h"
#include "FAPIDefs.h"



/*
 * Function definition: NPF_F_ATM_ConfigMgr_Register(
 *                          NPF_userContext_t userContext,
 *                          NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
 *                          NPF_callbackHandle_t *callbackHandle).
 */
NPF_error_t NPF_F_ATM_ConfigMgr_Register(
    NPF_userContext_t userContext,
    NPF_F_ATM_ConfigMgr_CallBackFunc_t callbackFunc,
    NPF_callbackHandle_t *callbackHandle)
{
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_Register(%d,..,..)\n",userContext);
    if(callbackFunc == NULL)
    {
       APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_Register - ERROR: Callback function is NULL!\n");
        return NPF_E_BAD_CALLBACK_FUNCTION; 
    }
    
    if(callbackHandle == NULL)
    {
       APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_Register - Callback Handle Is Null!\n");
        return NPF_E_UNKNOWN;
    }
    // Operation registerCallBack is called, a function pointer and a user context
    // are passed by value. The callbackHandle is passed by reference and will be
    // updated by the CallbackManager.
    NPF_error_t returnValue = CallBackManager::instance().RegisterCallBack(userContext, callbackFunc, callbackHandle);

    return returnValue;
}

/**
 * Completion Callback Deregistration Function.
 * This function is used by the application to de-register a pair of user context and
 * callback function. If there are any outstanding calls related to the de-registered
 * callback function, the callback function might be called for those outstanding
 * calls even after de-registration. NPF_F_ATM_ConfigMgr_Deregister() is a synchronous
 * function and has no completion callback associated with it.
 * @param callbackHandle - IN The unique identifier representing the pair of user
 *        context and callback function to be de-registered.
 * @return Possible return values are:
 * - NPF_NO_ERROR - De-registration completed successfully.
 * - NPF_E_UNKNOWN - an unknown error occurred in the implementation such that there
 *        is no eror code defined that is more appropriate or informative
 * - NPF_E_BAD_CALLBACK_HANDLE - The function does not recognize the callback handle.
 *        There is no effect to the registered callback functions.
 */
NPF_error_t NPF_F_ATM_ConfigMgr_Deregister(
    NPF_IN NPF_callbackHandle_t callbackHandle)
{
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_Deregister(%d)\n",callbackHandle);
    
    NPF_error_t returnValue = CallBackManager::instance().DeRegisterCallBack(callbackHandle);

    return returnValue;    
}
  
/**
 * Function Definition: NPF_F_ATM_ConfigMgr_IfSet(
 *                         NPF_callbackHandle_t cbHandle,
 *                         NPF_correlator_t     cbCorrelator,
 *                         NPF_errorReporting_t errorReporting,
 *                         NPF_FE_Handle_t       feHandle,
 *                         NPF_BlockId_t        blockId,
 *                         NPF_uint32_t         numEntries,
 *                         NPF_F_ATM_ConfigMgr_IfCfg_t   *cfgArray)
 */    
NPF_error_t NPF_F_ATM_ConfigMgr_IfSet(
    NPF_callbackHandle_t cbHandle,
    NPF_correlator_t     cbCorrelator,
    NPF_errorReporting_t errorReporting,
    NPF_FE_Handle_t       feHandle,
    NPF_BlockId_t        blockId,
    NPF_uint32_t         numEntries,
    NPF_F_ATM_ConfigMgr_IfCfg_t   *cfgArray)
{  
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_IfSet(%d,%d,%d,..,..,%d,..)\n",cbHandle, cbCorrelator, errorReporting,numEntries); 
     
    if((cbHandle > _ATM_FAPI_SIM_CB_HANDLE_MAX)||(CallBackManager::instance().RetrieveCallback(cbHandle)==0)) 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfSet - Invalid Callback Handle!\n");
        return NPF_E_BAD_CALLBACK_HANDLE;
    }
     
    if((!numEntries > 0)||(cfgArray == NULL))
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfSet - Number Of Entries = 0 or I/F Array = Null!\n");
        return NPF_E_UNKNOWN;   
    }
    
    // Initialise callback data structure
    NPF_F_ATM_ConfigMgr_CallbackData_t data;
    data.resp = new NPF_F_ATM_ConfigMgr_AsyncResponse_t[numEntries];
    
    bool error;
    bool errorReportValid;
    // Depending on the number of entries, loop through and add
    // them to the interface table maintained by the tableManager.
    error = TableManager::instance().AddATMIf(cfgArray, numEntries, data);
   
    switch(errorReporting)
    {
        case NPF_REPORT_ALL:  
            // Trigger a callback to the registered client callback function. no delete b4 return
            CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            errorReportValid = true;
        break;
        case NPF_REPORT_NONE:
            errorReportValid = true;
        break;
        case NPF_REPORT_ERRORS:
            // Trigger a callback to the registered client callback function.
            if(error == false)
            {
                CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            }
            errorReportValid = true;
        break;
        default:
            errorReportValid = false;
        break;
    }
       
    if(errorReportValid == true)
    {
        return NPF_NO_ERROR;
    }else 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfSet - Error Reporting Invalid!\n");
        return NPF_E_UNKNOWN;    
    }
}

/**
 * Delete an ATM Interface.
 * This function deletes one or more interfaces. All connections established on this
 * interface should be deleted before the interface is deleted.
 * @param cbhandle - IN The callback handle returned by NPF_F_ATM_ConfigMgr_Register()
 * @param cbCorrelator - IN A unique application invocation value that will be
 *        supplied to the asynchronous completion callback routine.
 * @param errorReporting - IN An indication of whether the application desires to
 *        receive an asynchronous completion callback for this function call.
 * @param feHandle - IN The FE Handle returned by NPF_F_ATM_topologyGetFEInfoList ()
 *        call.
 * @param blockId - IN The unique identification of the ATM Configuration Manager.
 * @param delContainedObjs - IN When set to NPF_TRUE indicates that all objects
 *        contained within the deleted interface should also be deleted.
 *        Examples of contained objects are VC links and VP links. The effect
 *        of this parameter propagates hierarchically to all contained objects.
 *        For example if an interface is deleted, all VP links on that
 *        interface are deleted along with any VC links contained in those VP
 *        links and if any of those VC links contained any AAL2 channels the
 *        AAL2 channels will also be deleted. If this parameter is set to
 *        NPF_FALSE, the function will return an error if there are objects
 *        contained within the interface being deleted.
 * @param numEntries - IN Number of interfaces to delete
 * @param delArray - IN Pointer to an array of interface handles of interfaces to
 *        delete
 * @return Possible return values are:
 * - NPF_NO_ERROR - The operation is in progress.
 * - NPF_E_UNKNOWN - Failure due to problems encountered when handling the input
 *        parameters.
 * - NPF_E_BAD_CALLBACK_HANDLE - The interfaces were not configured because the
 *        callback handle was invalid.
 * - - NPF_ATM_F_E_CMGR_FEATURE_NOT_SUPP - The requested feature is not supported
 * @callback A total of numEntries asynchronous (NPF_F_ATM_ConfigMgr_AsyncResponse_t)
 * responses are passed to the callback function, in one or more invocations. Each
 * response contains an interface ID in the objId member of the response structure and
 * a success code or a possible error code for that interface. If the function
 * invocation was successful an error code NPF_NO_ERROR is returned. The following
 * error codes could be returned:
 * - NPF_NO_ERROR - Operation successful.
 * - NPF_ATM_F_E_CONT_OBJS_EXIST - Specified interface could not be deleted as it has
 *        other objects like VP/VC links configured in it and the
 *        delContainedObjs parameter was set to NPF_FALSE.
 * - NPF_E_RESOURCE_NONEXISTENT - Specified interface does not exist
 */
NPF_error_t NPF_F_ATM_ConfigMgr_IfDelete(
    NPF_IN NPF_callbackHandle_t cbHandle,
    NPF_IN NPF_correlator_t cbCorrelator,
    NPF_IN NPF_errorReporting_t errorReporting,
    NPF_IN NPF_FE_Handle_t feHandle,
    NPF_IN NPF_BlockId_t blockId,
    NPF_IN NPF_boolean_t delContainedObjs,
    NPF_IN NPF_uint32_t numEntries,
    NPF_IN NPF_F_ATM_IfID_t *delArray)
{
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_IfDelete(%d,%d,%d,..,..,%d,..)\n",cbHandle, cbCorrelator, errorReporting,numEntries); 
     
    if((cbHandle > _ATM_FAPI_SIM_CB_HANDLE_MAX)||(CallBackManager::instance().RetrieveCallback(cbHandle)==0)) 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfDelete - Invalid Callback Handle!\n");
        return NPF_E_BAD_CALLBACK_HANDLE;
    }
     
    if((!numEntries > 0)||(delArray == NULL))
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfDelete - Number Of Entries = 0 or I/F Array = Null!\n");
        return NPF_E_UNKNOWN;   
    }   
    
    if(delContainedObjs == true)
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfDelete - Delete Sub Contained Objects Not Supported!\n");
        return NPF_ATM_F_E_CMGR_FEATURE_NOT_SUPP;   
    } 
    
    // Initialise callback data structure
    NPF_F_ATM_ConfigMgr_CallbackData_t data;
    data.resp = new NPF_F_ATM_ConfigMgr_AsyncResponse_t[numEntries];
    
    bool error;
    bool errorReportValid;
    // Depending on the number of entries, loop through and delete
    // them from the interface table maintained by the tableManager.
    error = TableManager::instance().DeleteIf(delArray, numEntries, data);
   
    switch(errorReporting)
    {
        case NPF_REPORT_ALL:  
            // Trigger a callback to the registered client callback function. no delete b4 return
            CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            errorReportValid = true;
        break;
        case NPF_REPORT_NONE:
            errorReportValid = true;
        break;
        case NPF_REPORT_ERRORS:
            // Trigger a callback to the registered client callback function.
            if(error == false)
            {
                CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            }
            errorReportValid = true;
        break;
        default:
            errorReportValid = false;
        break;
    }
       
    if(errorReportValid == true)
    {
        return NPF_NO_ERROR;
    }else 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_IfDelete - Error Reporting Invalid!\n");
        return NPF_E_UNKNOWN;    
    }
}

/**
 * Function Definition: NPF_F_ATM_ConfigMgr_VcSet(
 *                         NPF_callbackHandle_t cbHandle,
 *                         NPF_correlator_t     cbCorrelator,
 *                         NPF_errorReporting_t errorReporting,
 *                         NPF_FE_Handle_t       feHandle,
 *                         NPF_BlockId_t        blockId,
 *                         NPF_uint32_t         numEntries,
 *                         NPF_F_ATM_ConfigMgr_Vc_t  *cfgArray)
 */
NPF_error_t NPF_F_ATM_ConfigMgr_VcSet(
    NPF_callbackHandle_t cbHandle,
    NPF_correlator_t     cbCorrelator,
    NPF_errorReporting_t errorReporting,
    NPF_FE_Handle_t       feHandle,
    NPF_BlockId_t        blockId,
    NPF_uint32_t         numEntries,
    NPF_F_ATM_ConfigMgr_Vc_t  *cfgArray)
{
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_VcSet(%d,%d,%d,..,..,%d,..)\n",cbHandle, cbCorrelator, errorReporting,numEntries); 

    if((cbHandle > _ATM_FAPI_SIM_CB_HANDLE_MAX)||(CallBackManager::instance().RetrieveCallback(cbHandle)==0)) 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcSet - Invalid Callback Handle!\n");
        return NPF_E_BAD_CALLBACK_HANDLE;
    }


    if((!numEntries > 0)||(cfgArray == NULL))
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcSet - Number Of Entries = 0 or VC Array = Null!\n");
        return NPF_E_UNKNOWN;   
    }
    
    // Initialise callback data structure
    NPF_F_ATM_ConfigMgr_CallbackData_t data;
    data.resp = new NPF_F_ATM_ConfigMgr_AsyncResponse_t[numEntries];
    bool error;
    bool errorReportValid;
    
    // Depending on the number of entries, loop through and add
    // them to the vc table maintained by the tableManager.    
    error = TableManager::instance().AddATMVC(cfgArray, numEntries, data);
 
    switch(errorReporting)
    {
        case NPF_REPORT_ALL:  
            // Trigger a callback to the registered client callback function.
            CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            errorReportValid = true;
        break;
        case NPF_REPORT_NONE:
            errorReportValid = true;
        break;
        case NPF_REPORT_ERRORS:
            // Trigger a callback to the registered client callback function.
            if(error == false)
            {
                CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            }
            errorReportValid = true;
        break;
        default:
            errorReportValid = false;
        break;
    }
    
    if(errorReportValid == true)
    {
        return NPF_NO_ERROR;
    }else 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcSet - Error Reporting Invalid!\n");
        return NPF_E_UNKNOWN;    
    }
 
}

/**
 * Function Definition: NPF_error_t NPF_F_ATM_ConfigMgr_VcLinkXcSet (
 *                          NPF_callbackHandle_t  cbHandle,
 *                          NPF_correlator_t      cbCorrelator,
 *                          NPF_errorReporting_t  errorReporting,
 *                          NPF_FE_Handle_t        feHandle,
 *                          NPF_BlockId_t         blockId,
 *                          NPF_uint32_t          numEntries,
 *                          NPF_F_ATM_ConfigMgr_VcLinkXc_t *vcLinkXc) 
 */
NPF_error_t NPF_F_ATM_ConfigMgr_VcLinkXcSet (
    NPF_callbackHandle_t  cbHandle,
    NPF_correlator_t      cbCorrelator,
    NPF_errorReporting_t  errorReporting,
    NPF_FE_Handle_t        feHandle,
    NPF_BlockId_t         blockId,
    NPF_uint32_t          numEntries,
    NPF_F_ATM_ConfigMgr_VcLinkXc_t *vcLinkXc)
{
    APISimTrace(3,"\n\n");
    APISimTrace(3,"START OF A FAPI CALL THREAD\n");
    APISimTrace(3,"Trace Level 3: NPF_F_ATM_ConfigMgr_VcLinkXcSet(%d,%d,%d,..,..,%d,..)\n",cbHandle, cbCorrelator, errorReporting,numEntries); 
 
    if((cbHandle > _ATM_FAPI_SIM_CB_HANDLE_MAX)||(CallBackManager::instance().RetrieveCallback(cbHandle)==0)) 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcLinkXcSet - Invalid Callback Handle!\n");
        return NPF_E_BAD_CALLBACK_HANDLE;
    }   

    
    if((!numEntries > 0)||(vcLinkXc == NULL))
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcLinkXcSet - Number Of Entries = 0 or XC Array = Null!\n");
        return NPF_E_UNKNOWN;   
    }
    // Initialise callback data structure
    NPF_F_ATM_ConfigMgr_CallbackData_t data;
    data.resp = new NPF_F_ATM_ConfigMgr_AsyncResponse_t[numEntries];
    bool error;
    bool errorReportValid;
    // Depending on the number of entries, loop through and add
    // them to the vc table maintained by the tableManager.    
    error = TableManager::instance().AddATMXC(vcLinkXc, numEntries, data);
 
    switch(errorReporting)
    {
        case NPF_REPORT_ALL:  
            // Trigger a callback to the registered client callback function.
            CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            errorReportValid = true;
        break;
        case NPF_REPORT_NONE:
            errorReportValid = true;
        break;
        case NPF_REPORT_ERRORS:
            // Trigger a callback to the registered client callback function.
            if(error == false)
            {
                CallBackHandler::instance().AsyncCallback(cbHandle, cbCorrelator, data);
            }
            errorReportValid = true;
        break;
        default:
            errorReportValid = false;
        break;
    }  
    
    if(errorReportValid == true)
    {
        return NPF_NO_ERROR;
    }else 
    {
        APISimTrace(1,"Trace Level 1: NPF_F_ATM_ConfigMgr_VcLinkXcSet - Error Reporting Invalid!\n");
        return NPF_E_UNKNOWN;    
    } 
    
}

