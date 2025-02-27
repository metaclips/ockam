-- Switch to ACCOUNTADMIN role for full cleanup permissions
USE ROLE ACCOUNTADMIN;
USE DATABASE MSSQL_API_DB;
USE SCHEMA MSSQL_API_SCHEMA;

-- Drop Stored Procedures and Functions
DROP PROCEDURE IF EXISTS OCKAM_MSSQL_QUERY();
DROP PROCEDURE IF EXISTS OCKAM_MSSQL_EXECUTE();
DROP FUNCTION IF EXISTS _OCKAM_QUERY_MSSQL(STRING);
DROP FUNCTION IF EXISTS _OCKAM_MSSQL_EXECUTE(STRING);
DROP FUNCTION IF EXISTS OCKAM_MSSQL_INSERT(STRING, ARRAY);

-- Drop Service
DROP SERVICE IF EXISTS MSSQL_API_CLIENT;

-- Drop External Access Integrations
DROP INTEGRATION IF EXISTS OCKAM;
DROP INTEGRATION IF EXISTS OCSP;

-- Drop Network Rules
DROP NETWORK RULE IF EXISTS OCKAM_OUT;
DROP NETWORK RULE IF EXISTS OCSP_OUT;

-- Drop Image Repository
DROP IMAGE REPOSITORY IF EXISTS MSSQL_API_REPOSITORY;

-- Drop Schema
DROP SCHEMA IF EXISTS MSSQL_API_SCHEMA;

-- Drop Compute Pool
DROP COMPUTE POOL IF EXISTS MSSQL_API_CP;

-- Drop Warehouse
DROP WAREHOUSE IF EXISTS MSSQL_API_WH;

-- Drop Database
DROP DATABASE IF EXISTS MSSQL_API_DB;

-- Drop Role
DROP ROLE IF EXISTS MSSQL_API_ROLE;
