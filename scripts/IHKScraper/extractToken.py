import time
import json
import logging
import re
from pathlib import Path
from typing import Optional, Dict, Any
from dataclasses import dataclass

import requests
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import (
    TimeoutException, 
    NoSuchElementException, 
    WebDriverException
)


@dataclass
class IHKConfig:
    """Configuration settings for IHK API access."""
    base_url: str = "https://bildung.ihk.de"
    api_base_url: str = "https://service.ihk.de/anwender/anwender-api/v1"
    username: str = ""
    password: str = ""
    ihk_nummer: str = "108"
    organisation_nummer: str = "560628108"
    
    # Browser settings
    headless: bool = True
    window_size: str = "1920,1080"
    user_agent: str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36"


class IHKScraper:
    """
    Professional IHK token extraction and API interaction class.
    
    This class handles:
    - Selenium-based authentication
    - Bearer token extraction from network logs
    - Authenticated API requests
    - Error handling and logging
    """
    
    def __init__(self, config: IHKConfig):
        """
        Initialize the IHK scraper with configuration.
        
        Args:
            config: IHKConfig instance with authentication details
        """
        self.config = config
        self.driver: Optional[webdriver.Chrome] = None
        self.session: Optional[requests.Session] = None
        self.bearer_token: Optional[str] = None
        
        # Setup logging
        self._setup_logging()
        
    def _setup_logging(self) -> None:
        """Configure logging for the scraper."""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('ihk_scraper.log'),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger(__name__)
        
    def _setup_browser(self) -> webdriver.Chrome:
        """
        Configure and initialize Chrome WebDriver.
        
        Returns:
            Configured Chrome WebDriver instance
            
        Raises:
            WebDriverException: If browser initialization fails
        """
        options = Options()
        
        # Browser configuration
        if self.config.headless:
            options.add_argument("--headless")
        options.add_argument("--no-sandbox")
        options.add_argument("--disable-dev-shm-usage")
        options.add_argument("--disable-gpu")
        options.add_argument(f"--window-size={self.config.window_size}")
        options.add_argument(f"--user-agent={self.config.user_agent}")
        
        # Enable performance logging for token extraction
        options.set_capability("goog:loggingPrefs", {"performance": "ALL"})
        
        try:
            driver = webdriver.Chrome(options=options)
            self.logger.info("Chrome WebDriver initialized successfully")
            return driver
        except Exception as e:
            self.logger.error(f"Failed to initialize WebDriver: {e}")
            raise WebDriverException(f"Browser initialization failed: {e}")
    
    def _accept_cookies(self) -> None:
        """Accept cookie consent if present."""
        try:
            cookie_button = WebDriverWait(self.driver, 3).until(
                EC.element_to_be_clickable(("xpath", "//button[contains(., 'Alle akzeptieren')]"))
            )
            cookie_button.click()
            self.logger.info("Cookie consent accepted")
        except TimeoutException:
            self.logger.info("Cookie banner not found or already accepted")
    
    def _perform_login(self) -> None:
        """
        Perform login sequence on IHK platform.
        
        Raises:
            TimeoutException: If login elements are not found
            ValueError: If credentials are missing
        """
        if not self.config.username or not self.config.password:
            raise ValueError("Username and password must be provided")
        
        try:
            # Navigate to login
            login_link = WebDriverWait(self.driver, 10).until(
                EC.element_to_be_clickable(("xpath", "//a[contains(., 'Jetzt anmelden!')]"))
            )
            login_link.click()
            self.logger.info("Navigated to login page")
            
            # Enter username
            username_field = WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located(("id", "username"))
            )
            username_field.send_keys(self.config.username)
            
            # Continue to password page
            self.driver.find_element("id", "kc-login").click()
            
            # Enter password
            password_field = WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located(("id", "password"))
            )
            password_field.send_keys(self.config.password)
            self.driver.find_element("id", "kc-login").click()
            
            self.logger.info("Login sequence completed")
            
        except TimeoutException as e:
            self.logger.error(f"Login failed due to timeout: {e}")
            raise
        except Exception as e:
            self.logger.error(f"Login failed: {e}")
            raise
    
    def _extract_bearer_token(self) -> str:
        """
        Extract bearer token from browser performance logs.
        
        Returns:
            Bearer token string
            
        Raises:
            ValueError: If no token is found in logs
        """
        try:
            # Navigate to API page to trigger token usage
            self.driver.get(f"{self.config.base_url}/service/berufsbilder-check")
            time.sleep(5)  # Allow API calls to complete
            
            # Extract performance logs
            logs = self.driver.get_log("performance")
            log_data = json.dumps(logs)
            
            self.logger.info(f"Extracted {len(log_data)} characters of log data")
            
            # Search for bearer token pattern
            token_pattern = r'Bearer ([A-Za-z0-9\-_\.]+)'
            match = re.search(token_pattern, log_data)
            
            if match:
                token = match.group(1)
                self.logger.info(f"Bearer token extracted: {token[:15]}...{token[-15:]}")
                return token
            else:
                raise ValueError("No bearer token found in performance logs")
                
        except Exception as e:
            self.logger.error(f"Token extraction failed: {e}")
            raise
    
    def _setup_session(self) -> requests.Session:
        """
        Create authenticated requests session with browser cookies.
        
        Returns:
            Configured requests session
        """
        session = requests.Session()
        
        # Transfer cookies from browser to session
        for cookie in self.driver.get_cookies():
            session.cookies.set(cookie["name"], cookie["value"])
        
        self.logger.info("Requests session configured with browser cookies")
        return session
    
    def authenticate(self) -> None:
        """
        Complete authentication flow and extract bearer token.
        
        Raises:
            Various exceptions related to browser automation or authentication
        """
        try:
            self.driver = self._setup_browser()
            
            # Load homepage
            self.driver.get(self.config.base_url)
            self.logger.info("Homepage loaded")
            
            # Handle cookie consent
            self._accept_cookies()
            
            # Perform login
            self._perform_login()
            
            # Extract bearer token
            self.bearer_token = self._extract_bearer_token()
            
            # Setup requests session
            self.session = self._setup_session()
            
            self.logger.info("Authentication completed successfully")
            
        except Exception as e:
            self.logger.error(f"Authentication failed: {e}")
            self._save_error_screenshot()
            raise
    
    def _save_error_screenshot(self) -> None:
        """Save screenshot when error occurs."""
        try:
            if self.driver:
                screenshot_path = Path("error_screenshot.png")
                self.driver.save_screenshot(str(screenshot_path))
                self.logger.info(f"Error screenshot saved: {screenshot_path}")
        except Exception as e:
            self.logger.warning(f"Could not save screenshot: {e}")
    
    def api_request(self, endpoint: str, method: str = "GET", **kwargs) -> requests.Response:
        """
        Make authenticated API request to IHK service.
        
        Args:
            endpoint: API endpoint path
            method: HTTP method (GET, POST, etc.)
            **kwargs: Additional arguments for requests
            
        Returns:
            Response object
            
        Raises:
            ValueError: If not authenticated
            requests.RequestException: For API request errors
        """
        if not self.session or not self.bearer_token:
            raise ValueError("Must authenticate before making API requests")
        
        url = f"{self.config.api_base_url}/{endpoint}"
        
        headers = {
            "Accept": "application/json",
            "Authorization": f"Bearer {self.bearer_token}",
            "Referer": f"{self.config.base_url}/",
            "X-Bereich-Intern-Extern": "extern",
            "X-Ex-Abb": "false",
            "X-Ihk-Nummer": self.config.ihk_nummer,
            "X-Organisation-Nummer-Lang": self.config.organisation_nummer,
            "User-Agent": self.config.user_agent
        }
        
        # Merge with any provided headers
        if 'headers' in kwargs:
            headers.update(kwargs.pop('headers'))
        
        try:
            response = self.session.request(method, url, headers=headers, **kwargs)
            self.logger.info(f"API request to {endpoint}: Status {response.status_code}")
            return response
            
        except requests.RequestException as e:
            self.logger.error(f"API request failed: {e}")
            raise
    
    def get_berufsbilder_check(self) -> Dict[Any, Any]:
        """
        Get vocational training data from IHK API.
        
        Returns:
            JSON response data
            
        Raises:
            requests.HTTPError: If API returns error status
        """
        response = self.api_request("anwender/berufsbilder/check")
        
        if response.status_code == 200:
            self.logger.info("Successfully retrieved berufsbilder data")
            return response.json()
        else:
            self.logger.error(f"API error {response.status_code}: {response.text}")
            response.raise_for_status()
    
    def close(self) -> None:
        """Clean up resources."""
        if self.driver:
            self.driver.quit()
            self.logger.info("WebDriver closed")
        
        if self.session:
            self.session.close()
            self.logger.info("Requests session closed")


def main():
    """Main execution function."""
    # Configuration
    config = IHKConfig(
        username="your_username_here",
        password="your_password_here"
    )
    
    scraper = IHKScraper(config)
    
    try:
        # Authenticate and extract token
        scraper.authenticate()
        
        # Make API request
        data = scraper.get_berufsbilder_check()
        
        # Process results
        print("\n" + "="*50)
        print("API Response:")
        print("="*50)
        print(json.dumps(data, indent=2, ensure_ascii=False))
        
    except Exception as e:
        print(f"\n Error: {e}")
        
    finally:
        scraper.close()


if __name__ == "__main__":
    main()