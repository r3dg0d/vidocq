use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub category: String,
}

pub fn get_sites() -> Vec<Site> {
    vec![
        // Major Social Networks
        Site { name: "Twitter/X".to_string(), url: "https://twitter.com/{}".to_string(), category: "Social Network".to_string() },
        // Facebook removed - requires authentication, blocks automated checks
        Site { name: "Instagram".to_string(), url: "https://www.instagram.com/{}/".to_string(), category: "Social Network".to_string() },
        // LinkedIn removed - blocks automated checks with HTTP 999
        Site { name: "TikTok".to_string(), url: "https://www.tiktok.com/@{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Snapchat".to_string(), url: "https://www.snapchat.com/add/{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Pinterest".to_string(), url: "https://www.pinterest.com/{}/".to_string(), category: "Social Network".to_string() },
        Site { name: "Reddit".to_string(), url: "https://www.reddit.com/user/{}".to_string(), category: "Forum".to_string() },
        Site { name: "YouTube".to_string(), url: "https://www.youtube.com/@{}".to_string(), category: "Video".to_string() },
        Site { name: "Twitch".to_string(), url: "https://www.twitch.tv/{}".to_string(), category: "Gaming".to_string() },
        
        // Tech & Development
        Site { name: "GitHub".to_string(), url: "https://github.com/{}".to_string(), category: "Development".to_string() },
        Site { name: "GitLab".to_string(), url: "https://gitlab.com/{}".to_string(), category: "Development".to_string() },
        Site { name: "Bitbucket".to_string(), url: "https://bitbucket.org/{}/".to_string(), category: "Development".to_string() },
        Site { name: "Stack Overflow".to_string(), url: "https://stackoverflow.com/users/{}".to_string(), category: "Forum".to_string() },
        Site { name: "Dev.to".to_string(), url: "https://dev.to/{}".to_string(), category: "Blog".to_string() },
        Site { name: "Medium".to_string(), url: "https://medium.com/@{}".to_string(), category: "Blog".to_string() },
        // Hashnode removed - strict rate limiting (HTTP 429)
        Site { name: "HackerRank".to_string(), url: "https://www.hackerrank.com/{}".to_string(), category: "Development".to_string() },
        Site { name: "CodePen".to_string(), url: "https://codepen.io/{}".to_string(), category: "Development".to_string() },
        Site { name: "LeetCode".to_string(), url: "https://leetcode.com/{}/".to_string(), category: "Development".to_string() },
        Site { name: "Codeforces".to_string(), url: "https://codeforces.com/profile/{}".to_string(), category: "Development".to_string() },
        Site { name: "AtCoder".to_string(), url: "https://atcoder.jp/users/{}".to_string(), category: "Development".to_string() },
        Site { name: "Kaggle".to_string(), url: "https://www.kaggle.com/{}".to_string(), category: "Data Science".to_string() },
        Site { name: "Replit".to_string(), url: "https://replit.com/@{}".to_string(), category: "Development".to_string() },
        
        // Creative & Design
        Site { name: "DeviantArt".to_string(), url: "https://www.deviantart.com/{}".to_string(), category: "Art".to_string() },
        Site { name: "Behance".to_string(), url: "https://www.behance.net/{}".to_string(), category: "Design".to_string() },
        Site { name: "Dribbble".to_string(), url: "https://dribbble.com/{}".to_string(), category: "Design".to_string() },
        Site { name: "ArtStation".to_string(), url: "https://www.artstation.com/{}".to_string(), category: "Art".to_string() },
        Site { name: "Flickr".to_string(), url: "https://www.flickr.com/people/{}/".to_string(), category: "Photography".to_string() },
        Site { name: "500px".to_string(), url: "https://500px.com/p/{}".to_string(), category: "Photography".to_string() },
        Site { name: "Unsplash".to_string(), url: "https://unsplash.com/@{}".to_string(), category: "Photography".to_string() },
        
        // Forums & Communities
        Site { name: "Steam".to_string(), url: "https://steamcommunity.com/id/{}".to_string(), category: "Gaming".to_string() },
        Site { name: "Discord".to_string(), url: "https://discord.com/users/{}".to_string(), category: "Social".to_string() },
        Site { name: "Xbox Live".to_string(), url: "https://account.xbox.com/en-us/profile?gamertag={}".to_string(), category: "Gaming".to_string() },
        Site { name: "PlayStation".to_string(), url: "https://psnprofiles.com/{}".to_string(), category: "Gaming".to_string() },
        Site { name: "Wikipedia".to_string(), url: "https://en.wikipedia.org/wiki/User:{}".to_string(), category: "Wiki".to_string() },
        Site { name: "Wikia/Fandom".to_string(), url: "https://www.fandom.com/users/{}".to_string(), category: "Wiki".to_string() },
        Site { name: "Quora".to_string(), url: "https://www.quora.com/profile/{}".to_string(), category: "Forum".to_string() },
        Site { name: "Product Hunt".to_string(), url: "https://www.producthunt.com/@{}".to_string(), category: "Tech".to_string() },
        Site { name: "AngelList".to_string(), url: "https://angel.co/{}".to_string(), category: "Professional".to_string() },
        
        // Video Platforms
        Site { name: "Vimeo".to_string(), url: "https://vimeo.com/{}".to_string(), category: "Video".to_string() },
        Site { name: "Dailymotion".to_string(), url: "https://www.dailymotion.com/{}".to_string(), category: "Video".to_string() },
        
        // Music
        Site { name: "Spotify".to_string(), url: "https://open.spotify.com/user/{}".to_string(), category: "Music".to_string() },
        Site { name: "SoundCloud".to_string(), url: "https://soundcloud.com/{}".to_string(), category: "Music".to_string() },
        Site { name: "Last.fm".to_string(), url: "https://www.last.fm/user/{}".to_string(), category: "Music".to_string() },
        Site { name: "Bandcamp".to_string(), url: "https://{}.bandcamp.com".to_string(), category: "Music".to_string() },
        
        // News & Blogging
        Site { name: "Tumblr".to_string(), url: "https://{}.tumblr.com".to_string(), category: "Blog".to_string() },
        Site { name: "WordPress.com".to_string(), url: "https://{}.wordpress.com".to_string(), category: "Blog".to_string() },
        Site { name: "Blogger".to_string(), url: "https://{}.blogspot.com".to_string(), category: "Blog".to_string() },
        
        // Professional & Business
        Site { name: "Crunchbase".to_string(), url: "https://www.crunchbase.com/person/{}".to_string(), category: "Professional".to_string() },
        Site { name: "About.me".to_string(), url: "https://about.me/{}".to_string(), category: "Professional".to_string() },
        Site { name: "Keybase".to_string(), url: "https://keybase.io/{}".to_string(), category: "Social".to_string() },
        
        // Gaming Communities (Mixer was shut down in 2020 - removed)
        Site { name: "Roblox".to_string(), url: "https://www.roblox.com/user.aspx?username={}".to_string(), category: "Gaming".to_string() },
        Site { name: "Chess.com".to_string(), url: "https://www.chess.com/member/{}".to_string(), category: "Gaming".to_string() },
        Site { name: "Lichess".to_string(), url: "https://lichess.org/@/{}".to_string(), category: "Gaming".to_string() },
        
        // Coding & Tech Communities
        Site { name: "Gitee".to_string(), url: "https://gitee.com/{}".to_string(), category: "Development".to_string() },
        Site { name: "SourceForge".to_string(), url: "https://sourceforge.net/u/{}/profile".to_string(), category: "Development".to_string() },
        Site { name: "Launchpad".to_string(), url: "https://launchpad.net/~{}".to_string(), category: "Development".to_string() },
        Site { name: "FreeCodeCamp".to_string(), url: "https://www.freecodecamp.org/{}".to_string(), category: "Education".to_string() },
        
        // Additional Platforms
        Site { name: "VK".to_string(), url: "https://vk.com/{}".to_string(), category: "Social Network".to_string() },
        Site { name: "OK.ru".to_string(), url: "https://ok.ru/{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Telegram".to_string(), url: "https://t.me/{}".to_string(), category: "Social".to_string() },
        Site { name: "Weibo".to_string(), url: "https://weibo.com/{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Douban".to_string(), url: "https://www.douban.com/people/{}".to_string(), category: "Social Network".to_string() },
        
        // Forums & Message Boards
        Site { name: "XDA Developers".to_string(), url: "https://forum.xda-developers.com/m/{}.0".to_string(), category: "Forum".to_string() },
        Site { name: "Stack Exchange".to_string(), url: "https://stackexchange.com/users/{}".to_string(), category: "Forum".to_string() },
        // Ask.fm removed - DNS issues, site may be down
        
        // Additional Sites
        Site { name: "Imgur".to_string(), url: "https://imgur.com/user/{}".to_string(), category: "Image".to_string() },
        Site { name: "Giphy".to_string(), url: "https://giphy.com/{}".to_string(), category: "Image".to_string() },
        Site { name: "SlideShare".to_string(), url: "https://www.slideshare.net/{}".to_string(), category: "Professional".to_string() },
        Site { name: "Scribd".to_string(), url: "https://www.scribd.com/{}".to_string(), category: "Document".to_string() },
        Site { name: "Patreon".to_string(), url: "https://www.patreon.com/{}".to_string(), category: "Crowdfunding".to_string() },
        Site { name: "Kickstarter".to_string(), url: "https://www.kickstarter.com/profile/{}".to_string(), category: "Crowdfunding".to_string() },
        Site { name: "IndieGoGo".to_string(), url: "https://www.indiegogo.com/individuals/{}".to_string(), category: "Crowdfunding".to_string() },
        Site { name: "Gumroad".to_string(), url: "https://{}.gumroad.com".to_string(), category: "E-commerce".to_string() },
        Site { name: "Etsy".to_string(), url: "https://www.etsy.com/shop/{}".to_string(), category: "E-commerce".to_string() },
        Site { name: "eBay".to_string(), url: "https://www.ebay.com/usr/{}".to_string(), category: "E-commerce".to_string() },
        Site { name: "Goodreads".to_string(), url: "https://www.goodreads.com/{}".to_string(), category: "Books".to_string() },
        Site { name: "Letterboxd".to_string(), url: "https://letterboxd.com/{}".to_string(), category: "Movies".to_string() },
        Site { name: "Trakt".to_string(), url: "https://trakt.tv/users/{}".to_string(), category: "Movies".to_string() },
        Site { name: "MyAnimeList".to_string(), url: "https://myanimelist.net/profile/{}".to_string(), category: "Anime".to_string() },
        Site { name: "AniList".to_string(), url: "https://anilist.co/user/{}".to_string(), category: "Anime".to_string() },
        Site { name: "Fandom".to_string(), url: "https://www.fandom.com/user/{}".to_string(), category: "Wiki".to_string() },
        
        // More Development
        Site { name: "JSFiddle".to_string(), url: "https://jsfiddle.net/user/{}/".to_string(), category: "Development".to_string() },
        Site { name: "Pastebin".to_string(), url: "https://pastebin.com/u/{}".to_string(), category: "Development".to_string() },
        Site { name: "HackerEarth".to_string(), url: "https://www.hackerearth.com/@{}".to_string(), category: "Development".to_string() },
        Site { name: "TopCoder".to_string(), url: "https://www.topcoder.com/members/{}".to_string(), category: "Development".to_string() },
        Site { name: "Exercism".to_string(), url: "https://exercism.org/profiles/{}".to_string(), category: "Development".to_string() },
        Site { name: "Glitch".to_string(), url: "https://glitch.com/@{}".to_string(), category: "Development".to_string() },
        
        // More Social (MySpace removed - SSL certificate issues)
        Site { name: "Badoo".to_string(), url: "https://badoo.com/profile/{}".to_string(), category: "Dating".to_string() },
        Site { name: "Bumble".to_string(), url: "https://bumble.com/app/profile/{}".to_string(), category: "Dating".to_string() },
        
        // More Creative
        Site { name: "Mixcloud".to_string(), url: "https://www.mixcloud.com/{}/".to_string(), category: "Music".to_string() },
        // Spotify Artist removed - uses IDs, not usernames
        
        // More Professional
        Site { name: "Coursera".to_string(), url: "https://www.coursera.org/user/{}".to_string(), category: "Education".to_string() },
        Site { name: "Udemy".to_string(), url: "https://www.udemy.com/user/{}/".to_string(), category: "Education".to_string() },
        Site { name: "edX".to_string(), url: "https://www.edx.org/user/{}".to_string(), category: "Education".to_string() },
        
        // More Gaming
        Site { name: "Epic Games".to_string(), url: "https://www.epicgames.com/account/personal?productName=&lang=en".to_string(), category: "Gaming".to_string() },
        Site { name: "Battle.net".to_string(), url: "https://blizzard.com/invite/{}".to_string(), category: "Gaming".to_string() },
        Site { name: "Origin".to_string(), url: "https://www.origin.com/usa/en-us/profile/{}".to_string(), category: "Gaming".to_string() },
        Site { name: "Uplay".to_string(), url: "https://club.ubisoft.com/en-US/profile/{}".to_string(), category: "Gaming".to_string() },
        
        // Additional platforms to reach 100+
        Site { name: "Gravatar".to_string(), url: "https://en.gravatar.com/{}".to_string(), category: "Profile".to_string() },
        Site { name: "Disqus".to_string(), url: "https://disqus.com/by/{}/".to_string(), category: "Forum".to_string() },
        Site { name: "Slideshare".to_string(), url: "https://www.slideshare.net/{}".to_string(), category: "Professional".to_string() },
        Site { name: "Vero".to_string(), url: "https://vero.co/{}".to_string(), category: "Social Network".to_string() },
        // Ello removed - HTTP 520 Cloudflare errors, site may be down
        Site { name: "Mastodon".to_string(), url: "https://mastodon.social/@{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Bluesky".to_string(), url: "https://bsky.app/profile/{}.bsky.social".to_string(), category: "Social Network".to_string() },
        Site { name: "Threads".to_string(), url: "https://www.threads.net/@{}".to_string(), category: "Social Network".to_string() },
        Site { name: "Wattpad".to_string(), url: "https://www.wattpad.com/user/{}".to_string(), category: "Writing".to_string() },
        Site { name: "Archive of Our Own".to_string(), url: "https://archiveofourown.org/users/{}".to_string(), category: "Writing".to_string() },
        Site { name: "FanFiction".to_string(), url: "https://www.fanfiction.net/u/{}".to_string(), category: "Writing".to_string() },
    ]
}

