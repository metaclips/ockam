USE ROLE ACCOUNTADMIN;
USE DATABASE MSSQL_API_DB;
USE SCHEMA MSSQL_API_SCHEMA;

!set variable_substitution=true
!variables

-- Update VALUE_LIST with ockam egress details
CREATE OR REPLACE NETWORK RULE OCKAM_OUT
    TYPE = 'HOST_PORT'
    MODE = 'EGRESS'
    VALUE_LIST = ('&egress_list');

CREATE OR REPLACE EXTERNAL ACCESS INTEGRATION OCKAM
    ALLOWED_NETWORK_RULES = (OCKAM_OUT)
    ENABLED = true;

GRANT USAGE ON INTEGRATION OCKAM TO ROLE MSSQL_API_ROLE;

--- Describe the network rule to verify it was created correctly
DESCRIBE NETWORK RULE OCKAM_OUT;


USE ROLE MSSQL_API_ROLE;
USE DATABASE MSSQL_API_DB;
USE WAREHOUSE MSSQL_API_WH;
USE SCHEMA MSSQL_API_SCHEMA;


CREATE OR REPLACE NETWORK RULE OCSP_OUT
TYPE = 'HOST_PORT' MODE= 'EGRESS'
VALUE_LIST = ('ocsp.snowflakecomputing.com:80');

-- Create access integration

USE ROLE ACCOUNTADMIN;

GRANT CREATE INTEGRATION ON ACCOUNT TO ROLE MSSQL_API_ROLE;

CREATE OR REPLACE EXTERNAL ACCESS INTEGRATION OCSP
ALLOWED_NETWORK_RULES = (OCSP_OUT)
ENABLED = true;

GRANT USAGE ON INTEGRATION OCSP TO ROLE MSSQL_API_ROLE;


