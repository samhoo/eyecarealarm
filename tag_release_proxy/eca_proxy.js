// Cloudflare Worker 代码：用于安全代理私有仓库的 GitHub Release API

const GITHUB_OWNER = "samhoo";          // 你的 GitHub 用户名
const GITHUB_REPO = "eyecarealarm";     // 你的仓库名

// 在 Cloudflare 环境变量中配置你的 GitHub Token (推荐)
// 或者在这里直接写死 (测试可用，但生产环境建议走 Secret)
// const GITHUB_TOKEN = "你的GitHub_Personal_Access_Token";

addEventListener("fetch", event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  // 拼接 GitHub 官方的 latest release API 地址
  const targetUrl = `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`;

  // 从 Cloudflare 的环境变量中获取 Token（安全做法）
  // 也可以直接换成字符串: "Bearer ghp_xxxxxx"
  const token = ACCESS_TOKEN || (typeof GITHUB_TOKEN !== 'undefined' ? GITHUB_TOKEN : "");

  // 构造请求头发给 GitHub
  const headers = {
    "User-Agent": "Cloudflare-Worker-Proxy",
    "Accept": "application/vnd.github+json",
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  try {
    // 请求 GitHub API
    const response = await fetch(targetUrl, {
      method: "GET",
      headers: headers,
    });

    // 获取 GitHub 的响应内容
    const data = await response.text();

    // 将 GitHub 的响应原封不动返回给你的软件客户端，并加上允许跨域的 Header
    return new Response(data, {
      status: response.status,
      headers: {
        "Content-Type": "application/json;charset=UTF-8",
        "Access-Control-Allow-Origin": "*", // 允许任何客户端访问
      },
    });
  } catch (err) {
    return new Response(JSON.stringify({ error: err.message }), {
      status: 500,
      headers: { "Content-Type": "application/json" }
    });
  }
}