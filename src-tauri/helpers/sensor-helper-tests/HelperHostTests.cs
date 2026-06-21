using System.Text.Json;
using LibreHardwareMonitor.Hardware;
using PcHealth.SensorHelper;
using Xunit;

namespace PcHealth.SensorHelper.Tests;

public sealed class HelperHostTests
{
    [Fact]
    public void Run_reuses_one_sampler_for_two_samples_and_shuts_down()
    {
        var input = new StringReader(
            "{\"id\":1,\"command\":\"sample\"}\n" +
            "{\"id\":2,\"command\":\"sample\"}\n" +
            "{\"id\":3,\"command\":\"shutdown\"}\n");
        var output = new StringWriter();
        var sampler = new FakeSampler();

        new HelperHost(sampler, helperPid: 42).Run(input, output, TextWriter.Null);

        var responses = ParseResponses(output);
        Assert.Equal(3, responses.Count);
        Assert.Equal(42, responses[0].HelperPid);
        Assert.Equal(1, responses[0].SampleIndex);
        Assert.Equal(2, responses[1].SampleIndex);
        Assert.Equal("shutdown", responses[2].Command);
        Assert.Equal(2, sampler.SampleCount);
    }

    [Fact]
    public void Run_reports_malformed_json_and_continues_to_shutdown()
    {
        var input = new StringReader("not-json\n{\"id\":2,\"command\":\"shutdown\"}\n");
        var output = new StringWriter();

        new HelperHost(new FakeSampler(), helperPid: 42)
            .Run(input, output, TextWriter.Null);

        var responses = ParseResponses(output);
        Assert.False(responses[0].Ok);
        Assert.Null(responses[0].Id);
        Assert.Contains("protocol", responses[0].Command);
        Assert.Equal("shutdown", responses[1].Command);
    }

    [Fact]
    public void Run_reports_unknown_commands_without_stopping_the_host()
    {
        var input = new StringReader(
            "{\"id\":1,\"command\":\"unknown\"}\n" +
            "{\"id\":2,\"command\":\"sample\"}\n" +
            "{\"id\":3,\"command\":\"shutdown\"}\n");
        var output = new StringWriter();

        new HelperHost(new FakeSampler(), helperPid: 42)
            .Run(input, output, TextWriter.Null);

        var responses = ParseResponses(output);
        Assert.False(responses[0].Ok);
        Assert.Equal(1, responses[1].SampleIndex);
        Assert.Equal("shutdown", responses[2].Command);
    }

    [Fact]
    public void Normalize_keeps_missing_values_null_and_rejects_non_finite_values()
    {
        var errors = new List<string>();

        Assert.Null(SensorValues.Normalize(null, errors, "/cpu/temp/0"));
        Assert.Null(SensorValues.Normalize(float.NaN, errors, "/cpu/temp/1"));
        Assert.Null(SensorValues.Normalize(float.PositiveInfinity, errors, "/cpu/temp/2"));
        Assert.Equal(2, errors.Count);
    }

    [Theory]
    [InlineData(SensorType.Temperature, "C")]
    [InlineData(SensorType.Current, "A")]
    [InlineData(SensorType.Load, "%")]
    [InlineData(SensorType.Clock, "MHz")]
    [InlineData(SensorType.Frequency, "Hz")]
    [InlineData(SensorType.Power, "W")]
    [InlineData(SensorType.Fan, "RPM")]
    [InlineData(SensorType.Flow, "L/h")]
    [InlineData(SensorType.Voltage, "V")]
    [InlineData(SensorType.Level, "%")]
    [InlineData(SensorType.Factor, "1")]
    [InlineData(SensorType.SmallData, "MB")]
    [InlineData(SensorType.TimeSpan, "s")]
    [InlineData(SensorType.Timing, "ns")]
    [InlineData(SensorType.Energy, "mWh")]
    [InlineData(SensorType.Noise, "dBA")]
    [InlineData(SensorType.Conductivity, "uS/cm")]
    [InlineData(SensorType.Humidity, "%")]
    public void For_returns_explicit_units_for_known_sensor_types(
        SensorType sensorType,
        string expectedUnit)
    {
        Assert.Equal(expectedUnit, SensorUnits.For(sensorType));
    }

    private static List<HelperResponse> ParseResponses(StringWriter output)
    {
        return output.ToString()
            .Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries)
            .Select(line => JsonSerializer.Deserialize<HelperResponse>(line, ProtocolJson.Options)!)
            .ToList();
    }

    private sealed class FakeSampler : ICpuSensorSampler
    {
        public int SampleCount { get; private set; }

        public IReadOnlyList<HardwareRecord> Sample(ICollection<string> errors)
        {
            SampleCount += 1;
            return
            [
                new HardwareRecord(
                    "CPU",
                    "Cpu",
                    "/cpu/0",
                    null,
                    null,
                    [])
            ];
        }
    }
}
