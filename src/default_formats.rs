pub(crate) const REGISTER_FORMAT: &str = "%(ansify_if(
        ansify_if(justify(format_date(date), int(date_width)),
                  green if color and date > today),
                  bold if should_bold))
       %(ansify_if(
         ansify_if(justify(truncated(payee, int(payee_width)), int(payee_width)),
                   bold if color and !cleared and actual),
                   bold if should_bold))
       %(ansify_if(
         ansify_if(justify(truncated(display_account, int(account_width),
                                     int(abbrev_len)), int(account_width)),
                   blue if color),
                   bold if should_bold))
       %(ansify_if(
         justify(scrub(display_amount), int(amount_width),
                 3 + int(meta_width) + int(date_width) + int(payee_width)
                   + int(account_width) + int(amount_width) + int(prepend_width),
                 true, color),
                 bold if should_bold))
       %(ansify_if(
         justify(scrub(display_total), int(total_width),
                 4 + int(meta_width) + int(date_width) + int(payee_width)
                   + int(account_width) + int(amount_width) + int(total_width)
                   + int(prepend_width), true, color),
                 bold if should_bold))\n%/
      %(justify(\" \", int(date_width)))
       %(ansify_if(
         justify(truncated(has_tag(\"Payee\") ? payee : \" \",
                           int(payee_width)), int(payee_width)),
                   bold if should_bold))
       %$3 %$4 %$5\n";
pub(crate) const BALANCE_FORMAT: &str = "%(ansify_if(
  justify(scrub(display_total), 20,
          20 + int(prepend_width), true, color),
            bold if should_bold))
  %(!options.flat ? depth_spacer : \"\")
%-(ansify_if(
   ansify_if(partial_account(options.flat), blue if color),
             bold if should_bold))\n%/
%$1\n%/
%(prepend_width ? \" \" * int(prepend_width) : \"\")
--------------------\n";
