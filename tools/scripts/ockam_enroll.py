from selenium.webdriver.common.by import By
from selenium import webdriver
from selenium.webdriver.support.ui import WebDriverWait

import os

# Login credentials
username = "test@ockam.io"
password = "test"
activation_code = os.environ['ACTIVATION_CODE']

if len(activation_code) == 0:
    exit(1)

driver = webdriver.Chrome("chromedriver")

# head to activation page
driver.get("https://account.ockam.io/activate")

driver.find_element(By.NAME, "code").send_keys(activation_code)
driver.find_element(By.NAME, "action").click()

code = driver.find_element(
    By.XPATH, "/html/body/div/main/section/div/div/div/form/input[2]").get_attribute("value")

if code != activation_code:
    print("Activation code", activation_code,
          "is not same as code on browser", code)
    exit(1)

driver.find_element(
    By.XPATH, "/html/body/div/main/section/div/div/div/form/div[2]/button[1]").click()

driver.find_element(By.ID, "username").send_keys("ockam@test.com")
driver.find_element(By.ID, "password").send_keys('test@ockam.io0')
driver.find_element(By.NAME, "action").click()

# Find the success login page

# Close window
driver.close()
