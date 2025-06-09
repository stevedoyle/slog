/**
 * @file TableManager.cpp
 *
 * @date 22 March 2005
 *
 * @brief The TableManager maintains Maps relating to the configuration of the
 *        FWM product. 
 *
 * The TableManager class is a singleton. It stores ATM VC, ATM IF and ATM XC 
 * information in Maps. It co-ordinates the population of the Maps and retrieval
 * of Map entries.
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
#include "TableManager.h"
#include "pthread.h"
#include "APISimConfig.h"
#include "TraceMacro.h"


TableManager::TableManager()
{    
}

TableManager::~TableManager()
{
    VCIterator atmVCIter = m_ATMVCTable.begin();
    for(atmVCIter = m_ATMVCTable.begin(); atmVCIter != m_ATMVCTable.end(); atmVCIter++)
    {
        NPF_F_ATM_ConfigMgr_Vc_t vc = atmVCIter->second;
        if(vc.link_B != 0)delete [] vc.link_B;      
    }
    
    XCIterator atmXCIter = m_ATMXCTable.begin();
    for(atmXCIter = m_ATMXCTable.begin(); atmXCIter != m_ATMXCTable.end(); atmXCIter++)
    {
        NPF_F_ATM_ConfigMgr_VcLinkXc_t xc = atmXCIter->second;
        if(xc.link_B !=0)delete [] xc.link_B;
    }
}

TableManager& TableManager::instance()
{
    // Singleton Pattern
    static TableManager instance;
    return instance;
}

/**
 * Function Definition: addATMIf(NPF_F_ATM_ConfigMgr_IfCfg_t atmInterface)
 */
bool TableManager::AddATMIf(NPF_F_ATM_ConfigMgr_IfCfg_t* atmInterface, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
{  
    APISimTrace(3,"Trace Level 3: TableManager::AddATMIf(..,%d,..)\n",numEntries);
    data.type = NPF_F_ATM_CONFIGMGR_IF_SET;
    data.n_resp = 0;
    bool returnFlag = true;
    bool badInterfaceType = true;
    
    for(unsigned int x = 0; x < numEntries; x++)
    {
        data.n_resp += 1;
        badInterfaceType = false;
        
        if((atmInterface[x].ifType != NPF_F_ATM_IF_UNI)&&(atmInterface[x].ifType != NPF_F_ATM_IF_NNI))
        {
            APISimTrace(1,"Trace Level 1: TableManager::AddATMIf - Invalid Interface Type!\n");
            data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
            data.resp[(data.n_resp - 1)].objId.ifID = atmInterface[x].ifID;
            returnFlag = false;
            badInterfaceType = true;
        }
        
        if(badInterfaceType == false)
        {
            pthread_mutex_lock(&syncMutex);       
            InterfaceInsertPair insertReturn = m_ATMInterfaceTable.insert(InterfaceEntry(atmInterface[x].ifID, atmInterface[x]));        
            pthread_mutex_unlock(&syncMutex);
        
            if(!insertReturn.second)
            {
                // Entry already exists
                //cfen: do we support an interface been updated, if so should
                // we return a negative error?
                APISimTrace(1,"Trace Level 1: TableManager::AddATMIf - Interface Already Exists!\n");
                data.resp[(data.n_resp - 1)].error = NPF_E_RESOURCE_EXISTS; 
                data.resp[(data.n_resp - 1)].objId.ifID = atmInterface[x].ifID;
                returnFlag = false;  
            }else
            {
                data.resp[(data.n_resp - 1)].error = NPF_NO_ERROR;
                data.resp[(data.n_resp - 1)].objId.ifID = atmInterface[x].ifID;    
            }
        }
    }
    
    if(returnFlag == false)
    {
        data.allOK = NPF_FALSE;
        return false;
    }else
    {
        data.allOK = NPF_TRUE;
        return true;
    }
}

/**
 * Function Definition: DeleteIf(NPF_F_ATM_ConfigMgr_IfCfg_t atmInterface)
 */
bool TableManager::DeleteIf(NPF_F_ATM_IfID_t *delArray, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
{  
    APISimTrace(3,"Trace Level 3: TableManager::DeleteIf(..,%d,..)\n",numEntries);
    data.type = NPF_F_ATM_CONFIGMGR_IF_DELETE;
    data.n_resp = 0;
    bool returnFlag = true;
    bool removeInterface = true;
    int n;
    
    for(unsigned int x = 0; x < numEntries; x++)
    {
        removeInterface = true;
        data.n_resp += 1;
        n = 0;
        
        // Check if Interface has VC sub objects
        VCIterator subObjVCFindIter;
        
        pthread_mutex_lock(&syncMutex);
        for(subObjVCFindIter = m_ATMVCTable.begin(); subObjVCFindIter != m_ATMVCTable.end(); subObjVCFindIter++)
        {
            if(subObjVCFindIter->second.ifId == delArray[x])
            {
                APISimTrace(1,"Trace Level 1: TableManager::DeleteIf - Interface, %d, Has Sub Objects!\n",delArray[x]); 
                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_CONT_OBJS_EXIST; 
                data.resp[(data.n_resp - 1)].objId.ifID = delArray[x]; 
                returnFlag = false;
                removeInterface = false;
            }

        }
        pthread_mutex_lock(&syncMutex);
                
//        // Check if Interface had VP sub objects
//        VPIterator subObjVPFindIter;
//        
//        pthread_mutex_lock(&syncMutex);
//        for(subObjVPFindIter = m_ATMVPTable.begin(); subObjVPFindIter != m_ATMVPTable.end(); subObjVPFindIter++)
//        {
//            if(subObjVPFindIter->second.ifId == delArray[x])
//            {
//                APISimTrace(1,"Trace Level 1: TableManager::DeleteIf - Interface, %d, Has Sub Objects!\n",delArray[x]); 
//                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_CONT_OBJS_EXIST; 
//                data.resp[(data.n_resp - 1)].objId.ifID = delArray[x]; 
//                returnFlag = false;
//                removeInterface = false;
//            }
//
//        }
//        pthread_mutex_lock(&syncMutex);
    
        if(removeInterface == true)
        {
            pthread_mutex_lock(&syncMutex);    
            n = m_ATMInterfaceTable.erase(delArray[x]);   
            pthread_mutex_unlock(&syncMutex);
            
            if(n == 0)
            {
                APISimTrace(1,"Trace Level 1: TableManager::DeleteIf - Interface, %d, does not exist!\n",delArray[x]); 
                data.resp[(data.n_resp - 1)].error = NPF_E_RESOURCE_NONEXISTENT; 
                data.resp[(data.n_resp - 1)].objId.ifID = delArray[x]; 
                returnFlag = false;          
            }
            else
            {
                data.resp[(data.n_resp - 1)].error = NPF_NO_ERROR; 
                data.resp[(data.n_resp - 1)].objId.ifID = delArray[x];                
            }
        }
    }

    if(returnFlag == false)
    {
        data.allOK = NPF_FALSE;
        return false;
    }else
    {
        data.allOK = NPF_TRUE;
        return true;
    }    
}

/**
 * Function Definition: addATMVC(NPF_F_ATM_ConfigMgr_Vc_t atmVC, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
 */
bool TableManager::AddATMVC(NPF_F_ATM_ConfigMgr_Vc_t* atmVC, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
{
    //HAVE TO UPDATE Response structure to correspond with ITP
    
    
    APISimTrace(3,"Trace Level 3: TableManager::AddATMVC(..,%d,..)\n",numEntries);
    data.type = NPF_F_ATM_CONFIGMGR_VC_SET;
    data.n_resp = 0;
    bool returnFlag = true;
    bool vcErrored;
    NPF_error_t errorCode;

    for(unsigned int x = 0; x < numEntries; x++)
    {    
        vcErrored = false;
        // Check if interface exists
        pthread_mutex_lock(&syncMutex); 
          
        IFIterator findIF = m_ATMInterfaceTable.find(atmVC[x].ifId);

        pthread_mutex_unlock(&syncMutex);
        
        if(findIF == m_ATMInterfaceTable.end())
        {
            APISimTrace(1,"Trace Level 1: TableManager::AddATMVC - Interface Does Not Exist!\n");
            errorCode = NPF_E_UNKNOWN;
            vcErrored = true;
            returnFlag = false;
        }

        // Check if same vpi/vci pair exist under the specified interface in ATMVC entry
        if(vcErrored == false)
        {
            pthread_mutex_lock(&syncMutex);
            
            VCIterator checkAddressIFMatch;
            for(checkAddressIFMatch = m_ATMVCTable.begin(); checkAddressIFMatch != m_ATMVCTable.end(); checkAddressIFMatch++)
            {
                if(checkAddressIFMatch->second.ifId == atmVC[x].ifId)
                {
                    if((checkAddressIFMatch->second.vc.vci == atmVC[x].vc.vci)&&
                       (checkAddressIFMatch->second.vc.vpi == atmVC[x].vc.vpi))
                    {
                        APISimTrace(1,"Trace Level 1: TableManager::AddATMVC - Interface, VPI, VCI Entry Exists!\n");
                        errorCode = NPF_ATM_F_E_INVALID_VC_ADDRESS;
                        vcErrored = true;
                        returnFlag = false;           
                    } 
                }
            }
            pthread_mutex_unlock(&syncMutex);
        }
        
        // Check if there is a similar virt link entry existing in table
        if(vcErrored == false)
        {
            pthread_mutex_lock(&syncMutex);
            
            VCInsertPair insertReturn = m_ATMVCTable.insert(VCEntry(atmVC[x].vcLinkId, atmVC[x]));
            
           pthread_mutex_unlock(&syncMutex);
            
            if(!insertReturn.second)
            {
                APISimTrace(1,"Trace Level 1: TableManager::AddATMVC - Virtual Link Id Exists!\n");
                errorCode = NPF_ATM_F_E_INVALID_VC_ADDRESS; 
                vcErrored = true;
                returnFlag =  false;               
            }
        }       
        
        if(vcErrored == true)
        {
            data.n_resp += 1;
            data.allOK = NPF_FALSE;
            data.resp[(data.n_resp - 1)].error = errorCode; 
            data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmVC[x].vcLinkId;
            data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
        }
    }
    
    if(returnFlag == false)
    {
        return false;        
    }else
    {
        data.allOK = NPF_TRUE;
        return true;           
    }       
}

/**
 * Function Definition: addATMXC(NPF_F_ATM_ConfigMgr_VcLinkXc_t atmXC, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
 */
bool TableManager::AddATMXC(NPF_F_ATM_ConfigMgr_VcLinkXc_t* atmXC, NPF_uint32_t numEntries, NPF_F_ATM_ConfigMgr_CallbackData_t& data)
{
    //HAVE TO UPDATE Response structure to correspond with ITP
    
    APISimTrace(3,"Trace Level 3: TableManager::AddATMXC(..,%d,..)\n",numEntries);
    data.type = NPF_F_ATM_CONFIGMGR_VC_CROSSCONNECT_SET;
    data.n_resp = 0;
    bool returnFlag = true;
    bool xcErrored;
    
    for(unsigned int x = 0; x < numEntries; x++)
    {    
        xcErrored = false;
        // Check if the cross connect type is valid
        if(xcErrored == false)
        {          
            if((atmXC[x].link_B[0].xcType != NPF_F_ATM_EXT_TO_EXT)&&(atmXC[x].link_B[0].xcType != NPF_F_ATM_EXT_TO_INT)&&
            (atmXC[x].link_B[0].xcType != NPF_F_ATM_EXT_TO_BACK)&&(atmXC[x].link_B[0].xcType != NPF_F_ATM_BACK_TO_INT))
            {
                APISimTrace(1,"Trace Level 1: TableManager::AddATMXC - Invlaid Cross Connection Type!\n");
                data.n_resp += 1;
                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_A;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                xcErrored = true;
                returnFlag = false;        
            }
        }
        
        // Check to see if link A and link B match
        if(xcErrored ==false)
        {
            if(atmXC[x].link_A == atmXC[x].link_B[0].u.mapVcLink)
            {
                APISimTrace(1,"Trace Level 1: TableManager::AddATMXC - Link A and Link B Are The Same!\n");
                data.n_resp += 1;
                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_A;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                xcErrored = true;
                returnFlag = false;                
            }       
        }    
        
        // Check if link A exists and it is not part of any other cross connect
        pthread_mutex_lock(&syncMutex);
        
        VCIterator findLinkA = m_ATMVCTable.find(atmXC[x].link_A); 
        
        pthread_mutex_unlock(&syncMutex);
        
        if(xcErrored == false)
        {
            if(findLinkA == m_ATMVCTable.end())
            {
                APISimTrace(1,"Trace Level 1: TableManager::AddATMXC - Link A Does Not Exist!\n");
                data.n_resp += 1;
                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_A;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                xcErrored = true;
                returnFlag = false;     
            }else
            {
                if(findLinkA->second.numLink_B != 0)
                {
                    APISimTrace(1,"Trace Level 1: TableManager::AddATMXC - Link A Is Already Part Of A Cross Connect!\n");
                    data.n_resp += 1;
                    data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                    data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_A;
                    data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                    xcErrored = true;
                    returnFlag = false;
                }                
            }
        }
        
        // Check if link B exists and it is not part of any other cross connect
        pthread_mutex_lock(&syncMutex);
        
        VCIterator findLinkB = m_ATMVCTable.find(atmXC[x].link_B[0].u.mapVcLink); 
        
        pthread_mutex_unlock(&syncMutex);       
        if(xcErrored == false)
        {
    
            if(findLinkB == m_ATMVCTable.end())
            {
                APISimTrace(1,"Trace Level 1: TableManager::AddATMXC - Link B Does Not Exist!\n");
                data.n_resp += 1;
                data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_B[0].u.mapVcLink;
                data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                xcErrored = true;
                returnFlag = false;         
            }else
            {
                if(findLinkB->second.numLink_B != 0)
                {
                    data.n_resp += 1;
                    data.resp[(data.n_resp - 1)].error = NPF_ATM_F_E_INVALID_ATTRIBUTE;
                    data.resp[(data.n_resp - 1)].objId.linkId.atm_linkID_t = atmXC[x].link_B[0].u.mapVcLink;
                    data.resp[(data.n_resp - 1)].objId.linkId.atm_linkType_t = NPF_F_ATM_VC_LINK;
                    xcErrored = true;
                    returnFlag = false;                    
                }       
            }
        }
    
        // Check if XC already exists, if not add a new entry.
        if(xcErrored == false)
        {
            //atmXC.link_B = new NPF_F_ATM_ConfigMgr_VcLinkXcInfo_t[1];
            pthread_mutex_lock(&syncMutex);
            pair<map<unsigned int, NPF_F_ATM_ConfigMgr_VcLinkXc_t>::iterator, bool> insertReturn = m_ATMXCTable.insert(XCEntry(atmXC[x].link_B[0].vcXcId, atmXC[x]));
            
            pthread_mutex_unlock(&syncMutex);
            
            if(!insertReturn.second)
            {
                map<unsigned int, NPF_F_ATM_ConfigMgr_VcLinkXc_t>::iterator iter = insertReturn.first;
                data.n_resp += 1;
                data.resp[(data.n_resp - 1)].error = NPF_E_RESOURCE_EXISTS;
                data.resp[(data.n_resp - 1)].objId.vcXcId = iter->second.link_B[0].vcXcId;
                // this method shows that the entry exists, this may be modified
                // for new functionality as it is now obsolete
                // no point to multi point functionality   
                xcErrored = true;
                returnFlag = false;   
            }
        
    
            if(xcErrored == false)
            {
                pthread_mutex_lock(&syncMutex);
                
                insertReturn.first->second.link_B = new NPF_F_ATM_ConfigMgr_VcLinkXcInfo_t[1];
                insertReturn.first->second.link_B[0] = atmXC[x].link_B[0]; 
    
                findLinkA->second.link_B = new NPF_F_ATM_ConfigMgr_VcLinkXcInfo_t[1];
                findLinkA->second.numLink_B = 1;
                findLinkA->second.link_B[0].vcXcId = atmXC[x].link_B[0].vcXcId;
                findLinkA->second.link_B[0].xcType = NPF_F_ATM_EXT_TO_EXT;
                findLinkA->second.link_B[0].u.mapVcLink = atmXC[x].link_B[0].u.mapVcLink;
 
                findLinkB->second.link_B = new NPF_F_ATM_ConfigMgr_VcLinkXcInfo_t[1];   
                findLinkB->second.numLink_B = 1;
                findLinkB->second.link_B[0].vcXcId = atmXC[x].link_B[0].vcXcId;
                findLinkB->second.link_B[0].xcType = NPF_F_ATM_EXT_TO_EXT;
                findLinkB->second.link_B[0].u.mapVcLink = atmXC[x].link_A;
                
                pthread_mutex_unlock(&syncMutex);
            }
        }
    }
    
    if(returnFlag == false)
    {
        data.allOK = NPF_FALSE;
        return false;
    }else
    {
        data.allOK = NPF_TRUE;
        data.n_resp = 0;
        return true;
    }
}

//Test Operations for printing table contents
void TableManager::printIf()
{
    IFIterator atmIfIter = m_ATMInterfaceTable.begin();
    do{
        NPF_F_ATM_ConfigMgr_IfCfg_t interface = atmIfIter->second;
        printf("IfID: %d\n", interface.ifID);
        printf("IfType: %d\n", interface.ifType);
        printf("\n");
        printf("\n");
        atmIfIter++;
    }while(atmIfIter != m_ATMInterfaceTable.end());
}

void TableManager::printVc()
{
    VCIterator atmVCIter = m_ATMVCTable.begin();
    do{
        NPF_F_ATM_ConfigMgr_Vc_t vc = atmVCIter->second;
        printf("LinkID: %d\n",vc.vcLinkId);
        printf("VPI: %d\n", vc.vc.vpi);
        printf("VCI: %d\n", vc.vc.vci);
        printf("XC num link B: %d\n",vc.numLink_B);
        if(vc.link_B!=0)
        {
            printf("XC ID: %d\n", vc.link_B[0].vcXcId);
            printf("XC Type: %d\n", vc.link_B[0].xcType);
            printf("XC Link B: %d\n", vc.link_B[0].u.mapVcLink);      
        }
        printf("\n");
        printf("\n");
        atmVCIter++;
    }while(atmVCIter != m_ATMVCTable.end());
}

void TableManager::printXc()
{
    XCIterator atmXCIter = m_ATMXCTable.begin();
    do{
        NPF_F_ATM_ConfigMgr_VcLinkXc_t xc = atmXCIter->second;
        printf("Link A: %d\n",xc.link_A);
        printf("Num Link B: %d\n",xc.numLink_B);
        printf("XC ID: %d\n",xc.link_B[0].vcXcId);
        printf("XC Type: %d\n",xc.link_B[0].xcType);
        printf("Link B: %d\n",xc.link_B[0].u.mapVcLink);
      
        printf("\n");
        printf("\n");
        atmXCIter++;
    }while(atmXCIter != m_ATMXCTable.end());
}
