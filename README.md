# ~~in~~secure website

security is really too easy nowadays.

ever since [Let's Encrypt](https://letsencrypt.org/), sites on the web are pretty much secure by default, because everyone has HTTPS.

i mean, i only found one billion dollar company last June with a few million exposed records on a public site!

this is beside the point though. security is clearly too easy. but what if it wasn't?

[server name indication](https://en.wikipedia.org/wiki/Server_Name_Indication) is a method of making some things on the web easier. i assume. because i really do not know what it's used for except for the fact that it doesn't encrypt the subdomain you're accessing.

of course, the obvious next step is to communicate private information within this unencrypted field, just like the old days!

in this stupid website, you can login to your account by sending a request containing your credentials in the subdomain, such as:

`login-username-goldenstack-password-dosbox074-insecure.meow.i.ng`

this has the side effect of having a maximum username and password length. but it's fine because it's already an incredible secure website.

this site is a simple page where each user has a 5x5 grid of checkboxes that they can modify.

yeah, there's not much to it. but the point of this website is to be a monstrosity, not a streamlined site

i also made a website for it with my clearly incredible web design skills, in case you don't want to use raw requests.

please contact me if you would like to invest in my application

### \#\#\# fun facts

>  ip grabber resistant!

think this is an IP grabber? you can use this website entirely from common chat apps like Discord.

sending the URL in chat constitutes a valid request, and can be used to interact with this app.

relevant data will be returned and embedded in the message. 

> isn't this encrypted either way?

i lied. i actually do know what server name indication is. i just thought it would be funny to say i don't.

either way, i'm aware of DoH, and i'm happy to say that subdomains are still not encrypted under [DNS over HTTPS](https://support.mozilla.org/en-US/kb/firefox-dns-over-https)!

this means this website is still gloriously insecure.

unfortunately, [Encrypted Client Hello](https://support.mozilla.org/en-US/kb/understand-encrypted-client-hello) does actually encrypt this field. so please disable it in order to be able to fully experience this web site! 

> CORS-free!

ok, i lied again. there actually is CORS for the stylesheet and font:

- https://assets-font-insecure.meow.i.ng
- https://assets-css-insecure.meow.i.ng

but other than that, no CORS is needed! browsers send HEAD requests to confirm CORS, but this website doesn't care about which type of request is sent ! so it works either way.

also, most logic is handled via redirects anyway, so that isn't an issue either. 
