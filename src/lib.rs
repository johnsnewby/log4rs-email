//! Email appender for  log4rs.
use std::net::IpAddr;
use std;

extern crate gethostname;
#[macro_use]
extern crate lazy_static;
extern crate log4rs;
extern crate log;
extern crate lettre;
extern crate lettre_email;
extern crate mailin_embedded;

mod log4rs_email;

use mailin_embedded::*;
use std::io::Write;
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::net::Ipv4Addr;

    struct TestHandler {
        ip: IpAddr,
        domain: String,
        from: String,
        to: Vec<String>,
        is8bit: bool,
        expected_data: Vec<u8>,
        // Booleans set when callbacks are successful
        helo_called: bool,
        mail_called: bool,
        rcpt_called: bool,
        data_called: bool,
    }

    impl<'a> Handler for &'a mut TestHandler {
        fn helo(&mut self, ip: IpAddr, domain: &str) -> HeloResult {
            assert_eq!(self.ip, ip);
            assert_eq!(self.domain, domain);
            self.helo_called = true;
            HeloResult::Ok
        }

        // Called when a mail message is started
        fn mail(&mut self, ip: IpAddr, domain: &str, from: &str) -> MailResult {
            assert_eq!(self.ip, ip);
            assert_eq!(self.domain, domain);
            assert_eq!(self.from, from);
            self.mail_called = true;
            MailResult::Ok
        }

        // Called when a mail recipient is set
        fn rcpt(&mut self, to: &str) -> RcptResult {
            let valid_to = self.to.iter().any(|elem| elem == to);
            assert!(valid_to, "Invalid to address");
            self.rcpt_called = true;
            RcptResult::Ok
        }

        // Called to write an email message to a writer
        fn data(&mut self, domain: &str, from: &str, is8bit: bool, to: &[String]) -> DataResult {
            assert_eq!(self.domain, domain);
            assert_eq!(self.from, from);
            assert_eq!(self.to, to);
            assert_eq!(self.is8bit, is8bit);
            self.data_called = true;
            let test_writer = TestWriter {
                expected_data: self.expected_data.clone(),
                cursor: Cursor::new(Vec::with_capacity(80)),
            };
            DataResult::Ok(Box::new(test_writer))
        }
    }

    struct TestWriter {
        expected_data: Vec<u8>,
        cursor: Cursor<Vec<u8>>,
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.cursor.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.cursor.flush()
        }
    }

    impl Drop for TestWriter {
        fn drop(&mut self) {
            let actual_data = self.cursor.get_ref();
            assert_eq!(actual_data, &self.expected_data);
        }
    }

    #[test]
    fn callbacks() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let domain = "some.domain";
        let from = "ship@sea.com";
        let to = vec!["fish@sea.com".to_owned(), "seaweed@sea.com".to_owned()];
        let data = vec![
            b"Hello 8bit world \x40\x7f\r\n" as &[u8],
            b"Hello again\r\n" as &[u8],
        ];
        let mut expected_data = Vec::with_capacity(2);
        for line in data.clone() {
            expected_data.extend(line);
        }
        let mut handler = TestHandler {
            ip: ip.clone(),
            domain: domain.to_owned(),
            from: from.to_owned(),
            to: to.clone(),
            is8bit: true,
            expected_data,
            helo_called: false,
            mail_called: false,
            rcpt_called: false,
            data_called: false,
        };
        let mut session =
            smtp::SessionBuilder::new("server.domain").build(ip.clone(), &mut handler);
        let helo = format!("helo {}\r\n", domain).into_bytes();
        session.process(&helo);
        let mail = format!("mail from:<{}> body=8bitmime\r\n", from).into_bytes();
        session.process(&mail);
        let rcpt0 = format!("rcpt to:<{}>\r\n", &to[0]).into_bytes();
        let rcpt1 = format!("rcpt to:<{}>\r\n", &to[1]).into_bytes();
        session.process(&rcpt0);
        session.process(&rcpt1);
        session.process(b"data\r\n");
        for line in data {
            session.process(line);
        }
        session.process(b".\r\n");
        assert_eq!(handler.helo_called, true);
        assert_eq!(handler.mail_called, true);
        assert_eq!(handler.rcpt_called, true);
        assert_eq!(handler.data_called, true);
    }
}
