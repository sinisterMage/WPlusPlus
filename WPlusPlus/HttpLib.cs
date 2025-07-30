using System;
using System.Net.Http;
using System.Threading.Tasks;
using System.Collections.Generic;

public static class HttpLib
{
    private static readonly HttpClient client = new();

    public static async Task<string> Get(string url, Dictionary<string, string> headers = null)
    {
        var request = new HttpRequestMessage(HttpMethod.Get, url);

        if (headers != null)
        {
            foreach (var kv in headers)
                request.Headers.TryAddWithoutValidation(kv.Key, kv.Value);
        }

        var response = await client.SendAsync(request);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadAsStringAsync();
    }

    public static async Task<string> Post(string url, string body, Dictionary<string, string> headers = null)
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
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadAsStringAsync();
    }
}
