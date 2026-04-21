const http = require("http");

function weatherByCity(city) {
  const key = String(city || "").trim();
  if (!key) {
    return {
      city: "未知",
      temperature_c: 28,
      condition: "晴",
      summary: "未指定城市，默认晴，28°C。",
    };
  }
  const map = {
    深圳: { temperature_c: 29, condition: "多云" },
    北京: { temperature_c: 22, condition: "晴" },
    上海: { temperature_c: 26, condition: "小雨" },
    广州: { temperature_c: 30, condition: "阵雨" },
  };
  const item = map[key] || { temperature_c: 27, condition: "晴间多云" };
  return {
    city: key,
    temperature_c: item.temperature_c,
    condition: item.condition,
    summary: `${key}当前${item.condition}，气温 ${item.temperature_c}°C。`,
  };
}

const server = http.createServer((req, res) => {
  if (req.method !== "POST" || req.url !== "/mcp") {
    res.writeHead(404, { "content-type": "application/json; charset=utf-8" });
    res.end(JSON.stringify({ error: "not found" }));
    return;
  }
  let body = "";
  req.on("data", (chunk) => {
    body += chunk.toString("utf8");
  });
  req.on("end", () => {
    try {
      const payload = body ? JSON.parse(body) : {};
      const tool = String(payload.tool || "").trim();
      if (tool !== "get_weather") {
        res.writeHead(400, { "content-type": "application/json; charset=utf-8" });
        res.end(JSON.stringify({ error: "unknown tool" }));
        return;
      }
      const city = payload.params && payload.params.city;
      const result = weatherByCity(city);
      res.writeHead(200, { "content-type": "application/json; charset=utf-8" });
      res.end(JSON.stringify({ result }));
    } catch (err) {
      res.writeHead(500, { "content-type": "application/json; charset=utf-8" });
      res.end(JSON.stringify({ error: String(err) }));
    }
  });
});

server.listen(3456, "127.0.0.1", () => {
  // eslint-disable-next-line no-console
  console.log("weather-skill MCP demo running at http://127.0.0.1:3456/mcp");
});
