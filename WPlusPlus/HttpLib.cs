using System;
using System.Net.Http;
using System.Threading.Tasks;
using System.Collections.Generic;

public static class HttpLib
{
    private static readonly HttpClient client = new();

    public static async Task<WppHttpResponse> Get(string url, Dictionary<string, string>? headers = null)
    {
        var request = new HttpRequestMessage(HttpMethod.Get, url);

        if (headers != null)
        {
            foreach (var kv in headers)
                request.Headers.TryAddWithoutValidation(kv.Key, kv.Value);
        }

        var response = await client.SendAsync(request);
        var body = await response.Content.ReadAsStringAsync();
        var headerDict = new Dictionary<string, string>();

        foreach (var header in response.Headers)
            headerDict[header.Key] = string.Join(",", header.Value);

        foreach (var header in response.Content.Headers)
            headerDict[header.Key] = string.Join(",", header.Value);

        return new WppHttpResponse((int)response.StatusCode, body, headerDict);
    }

    public static async Task<WppHttpResponse> Post(string url, string body, Dictionary<string, string>? headers = null)
    {
        var request = new HttpRequestMessage(HttpMethod.Post, url)
        {
            Content = new StringContent(body)
        };

        if (headers != null)
        {
            foreach (var kv in headers)
                request.Headers.TryAddWithoutValidation(kv.Key, kv.Value);
        }

        var response = await client.SendAsync(request);
        var responseBody = await response.Content.ReadAsStringAsync();
        var headerDict = new Dictionary<string, string>();

        foreach (var header in response.Headers)
            headerDict[header.Key] = string.Join(",", header.Value);

        foreach (var header in response.Content.Headers)
            headerDict[header.Key] = string.Join(",", header.Value);

        return new WppHttpResponse((int)response.StatusCode, responseBody, headerDict);
    }
}


public class WppHttpResponse
{
    public int Status { get; set; }
    public string Body { get; set; }
    public Dictionary<string, string> Headers { get; set; }

    public WppHttpResponse(int status, string body, Dictionary<string, string> headers)
    {
        Status = status;
        Body = body;
        Headers = headers;
    }
}
