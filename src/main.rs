use std::time;
use std::sync::Arc;
use std::thread;
use std::io;
use std::fs;
use failure;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};

const HEADFUL: bool = false;

const EMAIL_INPUT: &'static str = "#email";
const LOGIN_BUTTON: &'static str = "#loginbutton";
const PASSWORD_INPUT: &'static str = "#pass";
const VIEW_MORE_COMMENTS: &'static str = "._4sxc._42ft";
const VIEW_ANSWERS: &'static str = "._5v47.fss";
const TOP_BAR_QUERY_SELECTOR: &'static str = "[role=\"banner\"]";
const THREAD_DIV_QUERY_SELECTOR: &'static str = "[role=feed]";

fn main() {
  let fb = browse_facebook();

  match fb {
    Ok(v) => {
      println!("Insert the name you want for the PDF file:");
      let file_name = ask_user_input();

      let path = format!("./output/{}.pdf", file_name);

      fs::create_dir("output").ok();

      fs::write(&path, &v)
          .expect(&format!("Unable to write to file: {}", path));

      println!("Program has finished it's job. Check your file in the /output folder!")
    },
    Err(e) => println!("Error: {:?}", e)
  }
}

fn browse_facebook() -> Result<Vec<u8>, failure::Error> {
  let browser = Browser::new(
    LaunchOptionsBuilder::default()
      .headless(!HEADFUL)
      .build()
      .unwrap(),
  )?;

  let tab = browser.wait_for_initial_tab()?;
  tab.navigate_to("https://www.facebook.com/")?;

  log_in(&tab)?;
  navigate_to_thread(&tab)?;

  if HEADFUL { tab.wait_for_element("._3ixn")?.click()?; }

  remove_elements_from_page(&tab)?;
  open_comments(&tab)?;
  open_view_more(&tab)?;
  Ok(generate_pdf(&tab)?)
}

fn log_in(tab: &Arc<Tab>) -> Result<(), failure::Error> {
  println!("Insert your e-mail");
  let login = ask_user_input();

  // Input e-mail
  let email_input = tab.wait_for_element(EMAIL_INPUT)?;
  email_input.type_into(&login).expect("Erro on login input");

  // Input password
  println!("Insert your password");
  let password = ask_user_input();

  let password_input = tab.wait_for_element(PASSWORD_INPUT)?;
  password_input
    .type_into(&password)
    .expect("Error on password input");
  println!("Logging in...");

  // Click the login button
  tab.wait_for_element(LOGIN_BUTTON)?
    .click()
    .expect("Login error");

  tab.wait_for_element("#userNav")
    .expect("Couldn't login. Are your credentials correct?");

  println!("Logged in!");
  Ok(())
}

fn navigate_to_thread(tab: &Arc<Tab>) -> Result<(), failure::Error> {
  println!("Insert the thread permalink:");
  // TODO: Validate if the permalink is valid.
  // TODO: Faster feedback.

  let thread = ask_user_input();
  tab.navigate_to(&thread)?.wait_until_navigated()?;
  println!("Ok! Now we're on {:?}!", thread);
  Ok(())
}

fn ask_user_input() -> String {
  let mut x = String::new();
  io::stdin().read_line(&mut x).expect("Failed to get console input");
  return x.trim().into();
}

fn remove_elements_from_page(tab: &Arc<Tab>) -> Result<(), failure::Error> {
  println!("Removing some elements from the page...");

  let expression = format!("const element = document.querySelector('{}');
            element.parentNode.removeChild(element);", TOP_BAR_QUERY_SELECTOR);

  tab.evaluate(&expression, false)?;

  println!("The elements are removed!");
  Ok(())
}

fn open_comments(tab: &Arc<Tab>) -> Result<(), failure::Error> {
  println!("Opening all comments...");

  let element = tab.wait_for_element(VIEW_MORE_COMMENTS);

  match element {
    Ok(element) => {
      element.click()?;
      thread::sleep(time::Duration::from_millis(800));
    },
    Err(_e) => {
      println!("Now we can see all comments!");
      return Ok(())
    }
  }

  open_comments(tab)?;
  Ok(())
}

fn open_view_more(tab: &Arc<Tab>) -> Result<(), failure::Error> {
  println!("Just wait a little more while I click in every single \"View More\" for you... :)");

  let elements = tab.wait_for_elements(VIEW_ANSWERS);

  match elements {
    Ok(elements) => {
      for element in &elements {
        element.click()?;
      }
    },
    Err(_e) => {
      println!("'Key! Done!");
      return Ok(())
    }
  }

  Ok(())
}

fn generate_pdf(tab: &Arc<Tab>) -> Result<Vec<u8>, failure::Error> {
  println!("Generating PDF...");

  let root_div = tab.wait_for_element(THREAD_DIV_QUERY_SELECTOR)?;
  let html = root_div.call_js_fn("function() { return this.innerHTML;}", false)?.value.unwrap();

  let expression = format!("document.write(`{}`)", html.to_string()); // this won't work in HEADFUL mode :/
  tab.evaluate(&expression, false)?;

  let local_pdf = tab.print_to_pdf(None)?;

  println!("PDF Generated!");
  Ok(local_pdf)
}