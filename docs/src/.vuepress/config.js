const { description } = require("../../package");

module.exports = {
  base: "/voyager/",
  /**
   * Ref：https://v1.vuepress.vuejs.org/config/#title
   */
  title: "Voyager",
  /**
   * Ref：https://v1.vuepress.vuejs.org/config/#description
   */
  description: description,

  /**
   * Extra tags to be injected to the page HTML `<head>`
   *
   * ref：https://v1.vuepress.vuejs.org/config/#head
   */
  head: [
    ["meta", { name: "theme-color", content: "#36a8bf" }],
    ["meta", { name: "apple-mobile-web-app-capable", content: "yes" }],
    [
      "meta",
      { name: "apple-mobile-web-app-status-bar-style", content: "black" },
    ],
  ],

  /**
   * Theme configuration, here is the default theme configuration for VuePress.
   *
   * ref：https://v1.vuepress.vuejs.org/theme/default-theme-config.html
   */
  themeConfig: {
    repo: "",
    editLinks: false,
    docsDir: "",
    editLinkText: "",
    lastUpdated: true,
    sidebarDepth: 2,
    sidebar: 
    [
      "/docs/introduction",
      "/docs/guide",
      "/docs/features",      
    ],

    nav: [
      // { text: "Rust", link: "https://docs.rs/anchor-lang/latest/anchor_lang/" },
      { text: "GitHub", link: "https://github.com/cmwaters/voyager" }
    ],
  },

  /**
   * Apply plugins，ref：https://v1.vuepress.vuejs.org/zh/plugin/
   */
  plugins: [
    "dehydrate",
    "@vuepress/plugin-back-to-top",
    "@vuepress/plugin-medium-zoom",
  ],
};
